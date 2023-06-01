mod active_list;
mod communicator;
mod count_by_dir;
mod deadlock_detection;
mod direction;
pub mod grid;
mod parameters;
pub(crate) mod site;
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
use self::count_by_dir::CountByDir;
pub use self::direction::DirectionIndex;
use self::direction::Directions;
use self::grid::Cell;
use self::grid::FaceArea;
use self::grid::ParticleType;
use self::grid::RemoteNeighbour;
use self::site::Site;
pub use self::task::RateData;
use self::task::Task;
use self::timestep_level::TimestepLevel;
use self::timestep_state::TimestepState;
use crate::chemistry::hydrogen_only::HydrogenOnly;
use crate::chemistry::hydrogen_only::HydrogenOnlySpecies;
use crate::chemistry::Chemistry;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::MpiWorld;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::components;
use crate::components::Density;
use crate::components::HeatingRate;
use crate::components::IonizedHydrogenFraction;
use crate::components::Source;
use crate::components::Timestep;
use crate::cosmology::Cosmology;
use crate::hash_map::HashMap;
use crate::io::output::parameters::is_desired_field;
use crate::io::output::parameters::OutputParameters;
use crate::particle::HaloParticles;
use crate::particle::ParticleId;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::simulation_plugin::SimulationTime;
use crate::units::Dimensionless;
use crate::units::SourceRate;
use crate::units::Temperature;
use crate::units::Time;

pub type Rate<C> = <C as Chemistry>::Photons;
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
        sim.add_startup_system_to_stage(StartupStages::ReadInput, initialize_directions_system)
            .add_startup_system_to_stage(
                StartupStages::InsertDerivedComponents,
                initialize_optional_components_system,
            )
            .add_derived_component::<IonizedHydrogenFraction>()
            .add_derived_component::<Source>()
            .add_derived_component::<components::Rate>()
            .add_derived_component::<Density>()
            .add_derived_component::<components::Temperature>()
            .insert_non_send_resource(Option::<Sweep<HydrogenOnly>>::None)
            .add_startup_system_to_stage(StartupStages::InitSweep, init_sweep_system)
            .add_system_to_stage(Stages::Sweep, run_sweep_system)
            .add_parameter_type::<SweepParameters>();
        if is_desired_field::<HeatingRate>(sim) {
            sim.add_derived_component::<HeatingRate>();
        }
        if is_desired_field::<Timestep>(sim) {
            sim.add_derived_component::<Timestep>();
        }
    }
}

#[derive(Resource)]
struct Sweep<C: Chemistry> {
    directions: Directions,
    cells: Cells,
    sites: Sites<C>,
    halo_levels: HashMap<ParticleId, TimestepLevel>,
    to_solve: PriorityQueue<Task>,
    to_send: DataByRank<Queue<RateData<C>>>,
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
        halo_ids: Vec<ParticleId>,
        max_timestep: Time,
        timestep_safety_factor: Dimensionless,
        parameters: &SweepParameters,
        world_size: usize,
        world_rank: Rank,
        chemistry: C,
    ) -> Sweep<C> {
        let initial_level = TimestepLevel(parameters.num_timestep_levels - 1);
        let communicator = SweepCommunicator::<C>::new();
        let timestep_state = TimestepState::new(max_timestep, parameters.num_timestep_levels);
        let halo_levels = halo_ids.into_iter().map(|id| (id, initial_level)).collect();
        Sweep {
            cells: Cells::new(cells, parameters.num_timestep_levels, initial_level),
            sites: Sites::<C>::new(sites, parameters.num_timestep_levels, initial_level),
            halo_levels,
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

    fn count_cells_global(&mut self, level: TimestepLevel) -> usize {
        let local_count = self.cells.enumerate_active(level).count();
        let mut count_communicator = MpiWorld::new();
        count_communicator.all_gather_sum(&CellCount(local_count))
    }

    fn get_cell_counts_per_level(&mut self) -> Vec<usize> {
        self.timestep_state
            .iter_all_levels()
            .map(|level| self.count_cells_global(level))
            .collect()
    }

    fn print_cell_counts(&mut self, cell_counts_per_level: &[usize]) {
        for level in self.timestep_state.iter_allowed_levels() {
            info!(
                "Sweep: {:>10} cells at level {:>2} ({:>10.1} yr)",
                cell_counts_per_level[level.0],
                level.0,
                self.timestep_state.timestep_at_level(level).in_years(),
            );
        }
    }

    pub fn run_sweeps(&mut self) -> Time {
        let counts = self.get_cell_counts_per_level();
        self.print_cell_counts(&counts);
        for level in self.timestep_state.iter_levels_in_sweep_order() {
            if counts[level.0] > 0 {
                self.current_level = level;
                self.single_sweep();
            }
        }
        let time_elapsed = self.timestep_state.current_max_timestep();
        self.timestep_state.advance_allowed_levels();
        self.update_timestep_levels();
        time_elapsed
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
                self.handle_local_neighbour(d.rate, d.dir, d.id);
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
            let mut num_missing_upwind = CountByDir::new(self.directions.len(), 0);
            for (dir_index, dir) in self.directions.enumerate() {
                for (face, neighbour) in cell.neighbours.iter() {
                    if !face.points_upwind(dir) || neighbour.is_boundary() {
                        continue;
                    }
                    let is_active = self.is_active(neighbour.unwrap_id());
                    if !is_active {
                        continue;
                    }
                    num_missing_upwind[dir_index] += 1;
                    if let ParticleType::Remote(neighbour) = neighbour {
                        self.to_receive_count[neighbour.rank] += 1;
                    }
                }
            }
            self.sites.get_mut(id).num_missing_upwind = num_missing_upwind;
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

    fn get_level(&self, id: ParticleId) -> TimestepLevel {
        if id.rank == self.communicator.rank() {
            self.cells.get_level(id)
        } else {
            self.halo_levels[&id]
        }
    }

    fn is_active(&self, id: ParticleId) -> bool {
        self.get_level(id).is_active(self.current_level)
    }

    fn get_outgoing_rate(&mut self, task: &Task) -> Rate<C> {
        let cell = &self.cells.get(task.id);
        let site = self.sites.get_mut(task.id);
        let source = site.source_per_direction_bin(&self.directions);
        let incoming_rate = site.incoming_total_rate[task.dir.0].clone() + source;
        self.chemistry.get_outgoing_rate(cell, site, incoming_rate)
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_rate = self.get_outgoing_rate(&task);
        let site = self.sites.get_mut(task.id);
        let outgoing_rate_correction =
            outgoing_rate.clone() - site.outgoing_total_rate[task.dir.0].clone();
        site.outgoing_total_rate[task.dir.0] = outgoing_rate;
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
                let rate_correction_this_cell =
                    outgoing_rate_correction.clone() * (effective_area / total_effective_area);
                match neighbour {
                    ParticleType::Local(neighbour_id) => self.handle_local_neighbour(
                        rate_correction_this_cell,
                        task.dir,
                        *neighbour_id,
                    ),
                    ParticleType::Remote(remote) => {
                        self.handle_remote_neighbour(&task, rate_correction_this_cell, remote)
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
        incoming_rate_correction: Rate<C>,
        dir: DirectionIndex,
        neighbour: ParticleId,
    ) {
        let (site, is_active) = self
            .sites
            .get_mut_and_active_state(neighbour, self.current_level);
        site.incoming_total_rate[*dir] += incoming_rate_correction;
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
        rate_correction: Rate<C>,
        remote: &RemoteNeighbour,
    ) {
        if self.is_active(remote.id) {
            let rate_data = RateData {
                dir: task.dir,
                rate: rate_correction,
                id: remote.id,
            };
            self.to_send[remote.rank].push(rate_data);
        }
    }

    fn update_chemistry(&mut self) {
        for (id, cell) in self.cells.enumerate_active(self.current_level) {
            let (level, site) = self.sites.get_mut_with_level(id);
            let timestep = self.timestep_state.timestep_at_level(level);
            let source = site.source_per_direction_bin(&self.directions);
            let rate = site.total_incoming_rate() + source;
            site.change_timescale =
                self.chemistry
                    .update_abundances(site, rate, timestep, cell.volume, cell.size);
        }
    }

    fn update_timestep_levels(&mut self) {
        for (id, level, site) in self.sites.enumerate_with_levels_mut() {
            let desired_timestep = self.timestep_safety_factor * site.change_timescale;
            let desired_level = self
                .timestep_state
                .get_desired_level_from_desired_timestep(desired_timestep);
            *level = desired_level;
            self.cells.set_level(id, desired_level);
        }
        self.sites.update_bins();
        self.cells.update_bins();
        self.communicate_levels();
    }

    fn communicate_levels(&mut self) {
        let mut levels_comm = ExchangeCommunicator::new();
        let mut data: DataByRank<Vec<TimestepLevelData>> =
            DataByRank::from_communicator(&levels_comm);
        for (id, level, cell) in self.cells.enumerate_with_levels() {
            for (_, neighbour) in cell.neighbours.iter() {
                if let ParticleType::Remote(neighbour) = neighbour {
                    data[neighbour.rank].push(TimestepLevelData { id, level });
                }
            }
        }
        for (_, levels) in levels_comm.exchange_all(data).iter() {
            for level_data in levels {
                self.halo_levels.insert(level_data.id, level_data.level);
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
        &components::Temperature,
        &Source,
    )>,
    haloes: HaloParticles<&ParticleId>,
    sweep_parameters: Res<SweepParameters>,
    world_rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
    mut solver: NonSendMut<Option<Sweep<HydrogenOnly>>>,
    cosmology: Res<Cosmology>,
) {
    let cells: HashMap<_, _> = cells_query
        .iter()
        .map(|(id, cell)| (*id, cell.clone()))
        .collect();
    let sites: HashMap<_, _> = sites_query
        .iter()
        .map(
            |(_, id, density, ionized_hydrogen_fraction, temperature, source)| {
                (
                    *id,
                    Site::<HydrogenOnly>::new(
                        &directions,
                        HydrogenOnlySpecies::new(**ionized_hydrogen_fraction, **temperature),
                        **density,
                        **source,
                    ),
                )
            },
        )
        .collect();
    let halo_ids: Vec<_> = haloes.iter().copied().collect();
    #[cfg(test)]
    assert!(!cells.is_empty() && !sites.is_empty());
    *solver = Some(Sweep::new(
        &directions,
        cells,
        sites,
        halo_ids,
        sweep_parameters.max_timestep,
        sweep_parameters.timestep_safety_factor,
        &sweep_parameters,
        **world_size,
        **world_rank,
        HydrogenOnly {
            rate_treshold: sweep_parameters.significant_rate_treshold,
            scale_factor: cosmology.scale_factor(),
            timestep_safety_factor: sweep_parameters.timestep_safety_factor,
        },
    ));
}

fn run_sweep_system(
    mut solver: NonSendMut<Option<Sweep<HydrogenOnly>>>,
    mut sites: Particles<(
        &ParticleId,
        &mut IonizedHydrogenFraction,
        &mut components::Temperature,
    )>,
    mut heating_rates: Particles<(&ParticleId, &mut HeatingRate)>,
    mut timesteps: Particles<(&ParticleId, &mut Timestep)>,
    mut time: ResMut<SimulationTime>,
) {
    let solver = (*solver).as_mut().unwrap();
    let time_elapsed = solver.run_sweeps();
    **time += time_elapsed;
    for (id, mut fraction, mut temperature) in sites.iter_mut() {
        let site = solver.sites.get(*id);
        **fraction = site.species.ionized_hydrogen_fraction;
        **temperature = site.species.temperature;
    }
    for (id, mut heating_rate) in heating_rates.iter_mut() {
        let site = solver.sites.get(*id);
        **heating_rate = site.species.heating_rate;
    }
    for (id, mut timestep) in timesteps.iter_mut() {
        let site = solver.sites.get(*id);
        **timestep = site.species.timestep;
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
            components::Temperature(Temperature::kelvins(1000.0)),
            Source(SourceRate::zero()),
        ));
    }
}

fn initialize_optional_components_system(
    mut commands: Commands,
    output_parameters: Res<OutputParameters>,
    local_particles: Query<Entity, With<LocalParticle>>,
) {
    if output_parameters.is_desired_field::<HeatingRate>() {
        for entity in local_particles.iter() {
            commands
                .entity(entity)
                .insert(HeatingRate(units::HeatingRate::zero()));
        }
    }
    if output_parameters.is_desired_field::<HeatingRate>() {
        for entity in local_particles.iter() {
            commands.entity(entity).insert(Timestep(Time::zero()));
        }
    }
}
