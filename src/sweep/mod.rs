mod active_list;
mod chemistry_solver;
mod communicator;
pub mod components;
mod count_by_dir;
mod direction;
mod parameters;
mod site;
mod task;
#[cfg(test)]
#[cfg(not(feature = "mpi"))]
mod tests;
pub mod timestep_level;

use bevy::prelude::*;
use bevy::utils::StableHashMap;
use mpi::traits::Equivalence;
pub use parameters::DirectionsSpecification;
pub use parameters::SweepParameters;

use self::active_list::ActiveList;
use self::chemistry_solver::Solver;
use self::components::IonizedHydrogenFraction;
use self::components::Source;
use self::count_by_dir::CountByDir;
pub use self::direction::DirectionIndex;
use self::direction::Directions;
use self::site::Site;
pub use self::task::FluxData;
use self::task::Task;
use self::timestep_level::TimestepLevel;
use crate::communication::CommunicationPlugin;
use crate::communication::Communicator;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::components::Density;
use crate::grid::Cell;
use crate::grid::FaceArea;
use crate::grid::Neighbour;
use crate::grid::RemoteNeighbour;
use crate::hydrodynamics::HaloParticle;
use crate::parameters::TimestepParameters;
use crate::particle::ParticleId;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::Dimensionless;
use crate::units::PhotonFlux;
use crate::units::SourceRate;
use crate::units::Time;
use crate::units::PROTON_MASS;

pub type SweepCommunicator<'a> = self::communicator::SweepCommunicator<'a>;

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;
type Queue<T> = Vec<T>;

type Cells = ActiveList<Cell>;
type Sites = ActiveList<Site>;

#[derive(Named)]
pub struct SweepPlugin;

#[derive(Equivalence, PartialEq, Eq, Hash)]
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
        .add_required_component::<IonizedHydrogenFraction>()
        .add_required_component::<Source>()
        .add_derived_component::<components::Flux>()
        .add_system_to_stage(SimulationStages::ForceCalculation, sweep_system)
        .add_system_to_stage(
            SimulationStages::ForceCalculation,
            communicate_levels_system.after(sweep_system),
        )
        .add_parameter_type::<SweepParameters>()
        .add_plugin(CommunicationPlugin::<FluxData>::default())
        .add_plugin(CommunicationPlugin::<TimestepLevelData>::exchange());
    }
}

struct Sweep<'a> {
    directions: Directions,
    cells: Cells,
    sites: Sites,
    levels: StableHashMap<ParticleId, TimestepLevel>,
    to_solve: PriorityQueue<Task>,
    to_send: DataByRank<Queue<FluxData>>,
    to_solve_count: CountByDir,
    to_receive_count: DataByRank<usize>,
    max_timestep: Time,
    current_level: TimestepLevel,
    flux_treshold: PhotonFlux,
    communicator: SweepCommunicator<'a>,
}

impl<'a> Sweep<'a> {
    fn run(
        directions: &Directions,
        cells: StableHashMap<ParticleId, Cell>,
        sites: StableHashMap<ParticleId, Site>,
        levels: StableHashMap<ParticleId, TimestepLevel>,
        max_timestep: Time,
        parameters: &SweepParameters,
        world_size: usize,
        world_rank: Rank,
        communicator: SweepCommunicator,
    ) -> Sites {
        for level in levels.values() {
            assert!(level.0 < parameters.num_timestep_levels);
        }
        let mut solver = Sweep {
            cells: Cells::new(cells, &levels),
            sites: Sites::new(sites, &levels),
            levels,
            to_solve: PriorityQueue::new(),
            to_send: DataByRank::from_size_and_rank(world_size, world_rank),
            directions: directions.clone(),
            to_solve_count: CountByDir::empty(),
            to_receive_count: DataByRank::empty(),
            max_timestep,
            current_level: TimestepLevel(0),
            flux_treshold: parameters.significant_flux_treshold,
            communicator,
        };
        for i in 0..(2usize.pow(parameters.num_timestep_levels as u32 - 1)) {
            solver.current_level = TimestepLevel::lowest_active_from_iteration(
                parameters.num_timestep_levels,
                i as u32,
            );
            solver.single_sweep();
        }
        solver.sites
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
        for (entity, cell) in self.cells.enumerate_active(self.current_level) {
            let mut site = self.sites.get_mut(*entity);
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
                    if let Neighbour::Remote(neighbour) = neighbour {
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
            .flat_map(|(dir_index, dir)| {
                self.cells
                    .enumerate_active(self.current_level)
                    .filter(|(_, cell)| {
                        // Importantly, the !face_points_upwind cannot
                        // be changed to face_points_downwind, because
                        // we need to be inclusive of all faces, even
                        // those that have zero dot product with the
                        // face normal.
                        cell.neighbours.iter().all(|(face, neighbour)| {
                            !face.points_upwind(dir)
                                || neighbour.is_boundary()
                                || !self.is_active(neighbour.unwrap_id())
                        })
                    })
                    .map(move |(id, _)| Task {
                        id: *id,
                        dir: dir_index,
                    })
            })
            .collect();
        tasks
    }

    fn is_active(&self, id: ParticleId) -> bool {
        self.levels[&id].is_active(self.current_level)
    }

    fn single_sweep(&mut self) {
        info!(
            "{:indent$}Sweeping {} cells at level {:?}",
            "",
            self.cells.enumerate_active(self.current_level).count(),
            self.current_level.0,
            indent = self.current_level.0 * 2,
        );
        self.init_counts();
        self.to_solve = self.get_initial_tasks();
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

    fn get_outgoing_flux(&mut self, task: &Task) -> PhotonFlux {
        let cell = &self.cells.get(task.id);
        let site = self.sites.get_mut(task.id);
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.ionized_hydrogen_fraction);
        let source = site.source_per_direction_bin(&self.directions);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let flux = site.incoming_total_flux[task.dir.0] + source;
        if flux < self.flux_treshold {
            PhotonFlux::zero()
        } else {
            let absorbed_fraction = (-neutral_hydrogen_number_density * sigma * cell.size).exp();
            flux * absorbed_fraction
        }
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = self.get_outgoing_flux(&task);
        let site = self.sites.get_mut(task.id);
        let outgoing_flux_correction = outgoing_flux - site.outgoing_total_flux[task.dir.0];
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
                    outgoing_flux_correction * (effective_area / total_effective_area);
                match neighbour {
                    Neighbour::Local(neighbour_id) => self.handle_local_neighbour(
                        flux_correction_this_cell,
                        task.dir,
                        *neighbour_id,
                    ),
                    Neighbour::Remote(remote) => {
                        self.handle_remote_neighbour(&task, flux_correction_this_cell, remote)
                    }
                    Neighbour::Boundary => {}
                }
            }
        }
    }

    fn handle_local_neighbour(
        &mut self,
        incoming_flux_correction: PhotonFlux,
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
        flux_correction: PhotonFlux,
        remote: &RemoteNeighbour,
    ) {
        let flux_data = FluxData {
            dir: task.dir,
            flux: flux_correction,
            id: remote.id,
        };
        self.to_send[remote.rank].push(flux_data);
    }

    fn update_chemistry(&mut self) {
        for (entity, cell) in self.cells.enumerate_active(self.current_level) {
            let (level, site) = self.sites.get_mut_with_level(*entity);
            let timestep = level.to_timestep(self.max_timestep);
            let source = site.source_per_direction_bin(&self.directions);
            let flux = site.total_incoming_flux() + source;
            site.ionized_hydrogen_fraction = Solver {
                ionized_hydrogen_fraction: site.ionized_hydrogen_fraction,
                timestep,
                density: site.density,
                volume: cell.volume,
                length: cell.size,
                flux,
            }
            .get_new_abundance();
        }
    }
}

pub fn sweep_system(
    directions: Res<Directions>,
    cells_query: Particles<(&ParticleId, &Cell)>,
    mut sites_query: Particles<(
        Entity,
        &ParticleId,
        &Density,
        &mut IonizedHydrogenFraction,
        Option<&Source>,
    )>,
    mut levels_query: HydroParticles<(&ParticleId, &mut TimestepLevel)>,
    timestep: Res<TimestepParameters>,
    sweep_parameters: Res<SweepParameters>,
    world_rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
    mut comm: Communicator<FluxData>,
) {
    let cells: StableHashMap<_, _> = cells_query
        .iter()
        .map(|(id, cell)| (*id, cell.clone()))
        .collect();
    let sites: StableHashMap<_, _> = sites_query
        .iter()
        .map(|(_, id, density, ionized_hydrogen_fraction, source)| {
            (
                *id,
                Site::new(
                    &directions,
                    **density,
                    **ionized_hydrogen_fraction,
                    source.map(|source| **source).unwrap_or(SourceRate::zero()),
                ),
            )
        })
        .collect();
    let levels: StableHashMap<_, _> = levels_query
        .iter()
        .map(|(id, level)| (*id, *level))
        .collect();
    #[cfg(test)]
    assert!(!cells.is_empty() && !sites.is_empty() && !levels.is_empty());
    let sites = Sweep::run(
        &directions,
        cells,
        sites,
        levels,
        timestep.max_timestep,
        &sweep_parameters,
        **world_size,
        **world_rank,
        SweepCommunicator::new(&mut comm),
    );
    for (entity, id, _, mut fraction, _) in sites_query.iter_mut() {
        let site = sites.get(*id);
        let new_fraction = site.ionized_hydrogen_fraction;
        let change_timescale =
            (**fraction / ((**fraction - new_fraction) / timestep.max_timestep)).abs();
        let desired_timestep = change_timescale * 0.01;
        let mut desired_level = TimestepLevel::from_max_timestep_and_desired_timestep(
            sweep_parameters.num_timestep_levels,
            timestep.max_timestep,
            desired_timestep,
        );
        let mut level = levels_query
            .get_component_mut::<TimestepLevel>(entity)
            .unwrap();
        if desired_level.0 + 1 < level.0 {
            // Never move down more than one level at a time
            desired_level.0 = level.0 - 1;
        }
        level.0 = desired_level.0;
        **fraction = new_fraction;
    }
}

fn communicate_levels_system(
    mut levels_comm: ExchangeCommunicator<TimestepLevelData>,
    mut halo_levels: HaloParticles<
        (Entity, &ParticleId, &mut TimestepLevel),
        Without<LocalParticle>,
    >,
    local_levels: Particles<(&ParticleId, &TimestepLevel, &Cell), Without<HaloParticle>>,
) {
    let mut data: DataByRank<Vec<TimestepLevelData>> = DataByRank::from_communicator(&*levels_comm);
    for (id, level, cell) in local_levels.iter() {
        for (_, n) in cell.neighbours.iter() {
            if let Neighbour::Remote(n) = n {
                data[n.rank].push(TimestepLevelData {
                    id: *id,
                    level: *level,
                });
            }
        }
    }
    let id_to_entity: StableHashMap<ParticleId, Entity> = halo_levels
        .iter()
        .map(|(entity, id, _)| (*id, entity))
        .collect();
    for (_, levels) in levels_comm.exchange_all(data).iter() {
        for level_data in levels {
            *halo_levels.get_mut(id_to_entity[&level_data.id]).unwrap().2 = level_data.level;
        }
    }
}

fn initialize_directions_system(mut commands: Commands, parameters: Res<SweepParameters>) {
    let directions: Directions = (&parameters.directions).into();
    commands.insert_resource(directions);
}

pub fn initialize_sweep_components_system(
    mut commands: Commands,
    local_particles: Query<Entity, With<LocalParticle>>,
    halo_particles: Query<Entity, With<HaloParticle>>,
    sweep_parameters: Res<SweepParameters>,
) {
    for entity in local_particles.iter() {
        commands.entity(entity).insert((
            Density(units::Density::zero()),
            components::IonizedHydrogenFraction(Dimensionless::zero()),
            TimestepLevel(sweep_parameters.num_timestep_levels - 1),
        ));
    }
    for entity in halo_particles.iter() {
        commands
            .entity(entity)
            .insert((TimestepLevel(sweep_parameters.num_timestep_levels - 1),));
    }
}
