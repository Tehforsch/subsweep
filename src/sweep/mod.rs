mod active_list;
mod chemistry;
mod chemistry_solver;
mod communicator;
pub mod components;
mod count_by_dir;
mod deadlock_detection;
mod direction;
mod parameters;
mod site;
mod task;
#[cfg(test)]
mod tests;
pub mod timestep_level;
mod timestep_state;

use bevy::prelude::*;
use derive_more::Into;
use mpi::traits::Equivalence;
pub use parameters::DirectionsSpecification;
pub use parameters::SweepParameters;

use self::active_list::ActiveList;
use self::chemistry::Chemistry;
use self::components::IonizedHydrogenFraction;
use self::components::Source;
use self::count_by_dir::CountByDir;
pub use self::direction::DirectionIndex;
use self::direction::Directions;
use self::site::Site;
pub use self::task::FluxData;
use self::task::Task;
use self::timestep_level::TimestepLevel;
use self::timestep_state::TimestepState;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::MpiWorld;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::components::Density;
use crate::grid::Cell;
use crate::grid::FaceArea;
use crate::grid::ParticleType;
use crate::grid::RemoteNeighbour;
use crate::hash_map::HashMap;
use crate::parameters::TimestepParameters;
use crate::particle::AllParticles;
use crate::particle::ParticleId;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::simulation_plugin::SimulationTime;
use crate::sweep::chemistry::HydrogenOnly;
use crate::sweep::chemistry::HydrogenOnlySpecies;
use crate::units::Dimensionless;
use crate::units::SourceRate;
use crate::units::Time;

pub type Flux<C> = <C as Chemistry>::Photons;
pub type Species<C> = <C as Chemistry>::Species;

pub type SweepCommunicator<C> = self::communicator::SweepCommunicator<C>;

#[derive(Equivalence, Clone, Into)]
pub struct CellCount(usize);

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;
type Queue<T> = Vec<T>;

type Cells = ActiveList<Cell>;
type Sites<C> = ActiveList<Site<C>>;

#[derive(Named)]
pub struct SweepPlugin;

#[derive(Debug, Equivalence, PartialEq, Eq, Hash)]
pub struct TimestepLevelData {
    level: TimestepLevel,
    id: ParticleId,
}

impl RaxiomPlugin for SweepPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(
            SimulationStartupStages::InsertComponents,
            initialize_directions_system,
        )
        .add_derived_component::<IonizedHydrogenFraction>()
        .add_derived_component::<Source>()
        .add_derived_component::<components::Flux>()
        .add_derived_component::<Density>()
        .insert_non_send_resource(Option::<Sweep<HydrogenOnly>>::None)
        .add_startup_system_to_stage(SimulationStartupStages::Sweep, init_sweep_system)
        .add_system_to_stage(SimulationStages::ForceCalculation, run_sweep_system)
        .add_parameter_type::<SweepParameters>();
    }
}

#[derive(Resource)]
struct Sweep<C: Chemistry> {
    directions: Directions,
    cells: Cells,
    sites: Sites<C>,
    levels: HashMap<ParticleId, TimestepLevel>,
    new_levels: HashMap<ParticleId, TimestepLevel>,
    to_solve: PriorityQueue<Task>,
    to_send: DataByRank<Queue<FluxData<C>>>,
    to_solve_count: CountByDir,
    to_receive_count: DataByRank<usize>,
    timestep_state: TimestepState,
    timestep_safety_factor: Dimensionless,
    current_level: TimestepLevel,
    communicator: SweepCommunicator<C>,
    check_deadlock: bool,
    chemistry: C,
}

impl<C: Chemistry> Sweep<C> {
    fn new(
        directions: &Directions,
        cells: HashMap<ParticleId, Cell>,
        sites: HashMap<ParticleId, Site<C>>,
        levels: HashMap<ParticleId, TimestepLevel>,
        max_timestep: Time,
        timestep_safety_factor: Dimensionless,
        parameters: &SweepParameters,
        world_size: usize,
        world_rank: Rank,
        chemistry: C,
    ) -> Sweep<C> {
        for level in levels.values() {
            assert!(level.0 < parameters.num_timestep_levels);
        }
        let communicator = SweepCommunicator::<C>::new();
        let timestep_state = TimestepState::new(max_timestep, parameters.num_timestep_levels);
        Sweep {
            cells: Cells::new(cells, &levels),
            sites: Sites::<C>::new(sites, &levels),
            levels,
            new_levels: HashMap::default(),
            to_solve: PriorityQueue::new(),
            to_send: DataByRank::from_size_and_rank(world_size, world_rank),
            directions: directions.clone(),
            to_solve_count: CountByDir::empty(),
            to_receive_count: DataByRank::empty(),
            timestep_safety_factor,
            timestep_state,
            current_level: TimestepLevel(0),
            communicator,
            check_deadlock: parameters.check_deadlock,
            chemistry,
        }
    }

    pub fn run_sweeps(&mut self) -> Time {
        self.print_cell_counts();
        for level in self.timestep_state.iter_levels_in_sweep_order() {
            self.current_level = level;
            self.single_sweep();
        }
        self.update_timestep_levels();
        let time_elapsed = self.timestep_state.current_max_timestep();
        self.timestep_state.advance_allowed_levels();
        time_elapsed
    }

    fn count_cells_global(&mut self, level: TimestepLevel) -> usize {
        let local_count = self.cells.enumerate_active(level).count();
        let mut count_communicator = MpiWorld::new();
        count_communicator.all_gather_sum(&CellCount(local_count))
    }

    pub fn print_cell_counts(&mut self) {
        for level in self.timestep_state.iter_allowed_levels() {
            let global_count = self.count_cells_global(level);
            info!("Sweep: {:>10} cells at level {:>2}", global_count, level.0);
        }
    }

    fn single_sweep(&mut self) {
        self.init_counts();
        self.to_solve = self.get_initial_tasks();
        if self.check_deadlock {
            self.check_deadlock();
        }
        self.solve();
        self.update_chemistry();
        for site in self.sites.iter() {
            debug_assert_eq!(site.num_missing_upwind.total(), 0);
        }
    }

    fn solve(&mut self) {
        while self.to_solve_count.total() > 0 || self.remaining_to_send_count() > 0 {
            if self.to_solve.is_empty() {
                self.receive_all_messages();
            }
            while let Some(task) = self.to_solve.pop() {
                self.solve_task(task);
            }
            self.send_all_messages();
        }
    }

    fn remaining_to_send_count(&self) -> usize {
        self.communicator.count_remaining_to_send()
    }

    fn receive_all_messages(&mut self) {
        for rank in self.communicator.other_ranks() {
            if self.to_receive_count[rank] > 0 {
                self.receive_messages_from_rank(rank);
            }
        }
    }

    fn receive_messages_from_rank(&mut self, rank: Rank) {
        let received = self.communicator.try_recv(rank);
        if let Some(received) = received {
            self.to_receive_count[rank] -= received.len();
            for d in received.into_iter() {
                self.handle_local_neighbour(d.flux, d.dir, d.id);
            }
        }
    }

    fn send_all_messages(&mut self) {
        self.communicator.try_send_all(&mut self.to_send);
    }

    pub fn init_counts(&mut self) {
        self.to_solve_count = CountByDir::new(
            self.directions.len(),
            self.cells.enumerate_active(self.current_level).count(),
        );
        self.to_receive_count = self
            .communicator
            .other_ranks()
            .into_iter()
            .map(|rank| (rank, 0))
            .collect();
        for (id, cell) in self.cells.enumerate_active(self.current_level) {
            let mut site = self.sites.get_mut(id);
            site.num_missing_upwind = CountByDir::new(self.directions.len(), 0);
            for (dir_index, dir) in self.directions.enumerate() {
                for (face, neighbour) in cell.neighbours.iter() {
                    if !face.points_upwind(dir) || neighbour.is_boundary() {
                        continue;
                    }
                    let is_active =
                        self.levels[&neighbour.unwrap_id()].is_active(self.current_level);
                    if !is_active {
                        continue;
                    }
                    site.num_missing_upwind[dir_index] += 1;
                    if let ParticleType::Remote(neighbour) = neighbour {
                        self.to_receive_count[neighbour.rank] += 1;
                    }
                }
            }
        }
    }

    fn get_initial_tasks(&self) -> PriorityQueue<Task> {
        let tasks = self
            .directions
            .enumerate()
            .flat_map(|(dir_index, _)| {
                self.sites
                    .enumerate_active(self.current_level)
                    .filter(move |(_, site)| site.num_missing_upwind[dir_index] == 0)
                    .map(move |(id, _)| Task { id, dir: dir_index })
            })
            .collect();
        tasks
    }

    fn is_active(&self, id: ParticleId) -> bool {
        if id.rank == self.communicator.rank() {
            self.cells.get_level(id).is_active(self.current_level)
        } else {
            self.levels[&id].is_active(self.current_level)
        }
    }

    fn get_outgoing_flux(&mut self, task: &Task) -> Flux<C> {
        let cell = &self.cells.get(task.id);
        let site = self.sites.get_mut(task.id);
        let source = site.source_per_direction_bin(&self.directions);
        let incoming_flux = site.incoming_total_flux[task.dir.0].clone() + source;
        self.chemistry.get_outgoing_flux(cell, site, incoming_flux)
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = self.get_outgoing_flux(&task);
        let site = self.sites.get_mut(task.id);
        let outgoing_flux_correction =
            outgoing_flux.clone() - site.outgoing_total_flux[task.dir.0].clone();
        site.outgoing_total_flux[task.dir.0] = outgoing_flux;
        let cell = &self.cells.get(task.id);
        self.to_solve_count.reduce(task.dir);
        // This is very inefficient, let's see if this ever becomes a bottleneck
        let neighbours = cell.neighbours.clone();
        let total_effective_area: FaceArea = cell
            .iter_downwind_faces(&self.directions[task.dir])
            .map(|face| face.area * face.normal.dot(*self.directions[task.dir]))
            .sum();
        for (face, neighbour) in neighbours.iter() {
            if face.points_downwind(&self.directions[task.dir]) {
                let effective_area = face.area * face.normal.dot(*self.directions[task.dir]);
                let flux_correction_this_cell =
                    outgoing_flux_correction.clone() * (effective_area / total_effective_area);
                match neighbour {
                    ParticleType::Local(neighbour_id) => self.handle_local_neighbour(
                        flux_correction_this_cell,
                        task.dir,
                        *neighbour_id,
                    ),
                    ParticleType::Remote(remote) => {
                        self.handle_remote_neighbour(&task, flux_correction_this_cell, remote)
                    }
                    ParticleType::Boundary => {}
                    ParticleType::LocalPeriodic(_) => {}
                    ParticleType::RemotePeriodic(_) => {}
                }
            }
        }
    }

    fn handle_local_neighbour(
        &mut self,
        incoming_flux_correction: Flux<C>,
        dir: DirectionIndex,
        neighbour: ParticleId,
    ) {
        let (site, is_active) = self
            .sites
            .get_mut_and_active_state(neighbour, self.current_level);
        site.incoming_total_flux[*dir] += incoming_flux_correction;
        if is_active {
            let num_remaining = site.num_missing_upwind.reduce(dir);
            if num_remaining == 0 {
                self.to_solve.push(Task { dir, id: neighbour })
            }
        }
    }

    fn handle_remote_neighbour(
        &mut self,
        task: &Task,
        flux_correction: Flux<C>,
        remote: &RemoteNeighbour,
    ) {
        if self.is_active(remote.id) {
            let flux_data = FluxData {
                dir: task.dir,
                flux: flux_correction,
                id: remote.id,
            };
            self.to_send[remote.rank].push(flux_data);
        }
    }

    fn update_chemistry(&mut self) {
        for (id, cell) in self.cells.enumerate_active(self.current_level) {
            let (level, site) = self.sites.get_mut_with_level(id);
            let timestep = self.timestep_state.timestep_at_level(level);
            let source = site.source_per_direction_bin(&self.directions);
            let flux = site.total_incoming_flux() + source;
            let change_timescale =
                self.chemistry
                    .update(site, flux, timestep, cell.volume, cell.size);
            let desired_timestep = self.timestep_safety_factor * change_timescale;
            let desired_level = self
                .timestep_state
                .get_desired_level_from_desired_timestep(desired_timestep);
            self.new_levels.insert(id, desired_level);
        }
    }

    fn update_timestep_levels(&mut self) {
        self.cells.update_levels(&self.new_levels);
        self.sites.update_levels(&self.new_levels);
        self.levels.extend(self.new_levels.drain());
        self.communicate_levels();
    }

    fn communicate_levels(&mut self) {
        let mut levels_comm = ExchangeCommunicator::new();
        let mut data: DataByRank<Vec<TimestepLevelData>> =
            DataByRank::from_communicator(&levels_comm);
        for (id, level, cell) in self.cells.enumerate_with_levels() {
            for (_, neighbour) in cell.neighbours.iter() {
                if let ParticleType::Remote(neighbour) = neighbour {
                    data[neighbour.rank].push(TimestepLevelData { id, level: level });
                }
            }
        }
        for (_, levels) in levels_comm.exchange_all(data).iter() {
            for level_data in levels {
                self.levels.insert(level_data.id, level_data.level);
            }
        }
    }
}

fn init_sweep_system(
    directions: Res<Directions>,
    cells_query: Particles<(&ParticleId, &Cell)>,
    sites_query: Particles<(
        Entity,
        &ParticleId,
        &Density,
        &IonizedHydrogenFraction,
        &Source,
    )>,
    levels_query: AllParticles<&ParticleId>,
    timestep: Res<TimestepParameters>,
    sweep_parameters: Res<SweepParameters>,
    world_rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
    mut solver: NonSendMut<Option<Sweep<HydrogenOnly>>>,
) {
    let cells: HashMap<_, _> = cells_query
        .iter()
        .map(|(id, cell)| (*id, cell.clone()))
        .collect();
    let sites: HashMap<_, _> = sites_query
        .iter()
        .map(|(_, id, density, ionized_hydrogen_fraction, source)| {
            (
                *id,
                Site::<HydrogenOnly>::new(
                    &directions,
                    HydrogenOnlySpecies {
                        ionized_hydrogen_fraction: **ionized_hydrogen_fraction,
                    },
                    **density,
                    **source,
                ),
            )
        })
        .collect();
    let levels: HashMap<_, _> = levels_query
        .iter()
        .map(|id| (*id, TimestepLevel(sweep_parameters.num_timestep_levels - 1)))
        .collect();
    #[cfg(test)]
    assert!(!cells.is_empty() && !sites.is_empty() && !levels.is_empty());
    *solver = Some(Sweep::new(
        &directions,
        cells,
        sites,
        levels,
        timestep.max_timestep,
        sweep_parameters.timestep_safety_factor,
        &sweep_parameters,
        **world_size,
        **world_rank,
        HydrogenOnly {
            flux_treshold: sweep_parameters.significant_flux_treshold,
        },
    ));
}

fn run_sweep_system(
    mut solver: NonSendMut<Option<Sweep<HydrogenOnly>>>,
    mut sites_query: Particles<(&ParticleId, &mut IonizedHydrogenFraction)>,
    mut time: ResMut<SimulationTime>,
) {
    let solver = (*solver).as_mut().unwrap();
    let time_elapsed = solver.run_sweeps();
    **time += time_elapsed;
    for (id, mut fraction) in sites_query.iter_mut() {
        let site = solver.sites.get(*id);
        **fraction = site.species.ionized_hydrogen_fraction;
    }
}

fn initialize_directions_system(mut commands: Commands, parameters: Res<SweepParameters>) {
    let directions: Directions = (&parameters.directions).into();
    commands.insert_resource(directions);
}

pub fn initialize_sweep_components_system(
    mut commands: Commands,
    local_particles: Query<Entity, With<LocalParticle>>,
) {
    for entity in local_particles.iter() {
        commands.entity(entity).insert((
            Density(units::Density::zero()),
            components::IonizedHydrogenFraction(Dimensionless::zero()),
            Source(SourceRate::zero()),
        ));
    }
}
