mod active_list;
mod chemistry_solver;
pub mod components;
mod count_by_dir;
mod direction;
mod parameters;
mod site;
mod task;
#[cfg(test)]
mod tests;
pub mod timestep_level;

use bevy::prelude::*;
use bevy::utils::HashMap;
pub use parameters::SweepParameters;

use self::active_list::ActiveList;
use self::chemistry_solver::solve_chemistry;
use self::components::IonizedHydrogenFraction;
use self::components::Source;
use self::count_by_dir::CountByDir;
use self::direction::Directions;
use self::site::Site;
use self::task::Task;
use self::timestep_level::TimestepLevel;
use crate::components::Density;
use crate::components::Position;
use crate::grid::Cell;
use crate::grid::FaceArea;
use crate::grid::Neighbour;
use crate::grid::RemoteNeighbour;
use crate::parameters::TimestepParameters;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::Amount;
use crate::units::PhotonFlux;
use crate::units::SourceRate;
use crate::units::Time;
use crate::units::PROTON_MASS;

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;

type Cells = ActiveList<Cell>;
type Sites = ActiveList<Site>;

#[derive(Named)]
pub struct SweepPlugin;

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
        .add_parameter_type::<SweepParameters>();
    }
}

struct Sweep {
    directions: Directions,
    cells: Cells,
    sites: Sites,
    to_solve: PriorityQueue<Task>,
    remaining_to_solve_count: CountByDir,
    max_timestep: Time,
    current_level: TimestepLevel,
}

impl Sweep {
    fn run(
        directions: &Directions,
        cells: HashMap<Entity, Cell>,
        sites: HashMap<Entity, Site>,
        levels: HashMap<Entity, TimestepLevel>,
        max_timestep: Time,
        num_timestep_levels: usize,
    ) -> Sites {
        assert!(cells.len() == sites.len());
        for level in levels.values() {
            assert!(level.0 < num_timestep_levels);
        }
        let mut solver = Sweep {
            cells: Cells::new(cells, &levels),
            sites: Sites::new(sites, &levels),
            to_solve: PriorityQueue::new(),
            directions: directions.clone(),
            remaining_to_solve_count: CountByDir::empty(),
            max_timestep,
            current_level: TimestepLevel(0),
        };
        for i in 0..(2usize.pow(num_timestep_levels as u32 - 1)) {
            solver.current_level =
                TimestepLevel::lowest_active_from_iteration(num_timestep_levels, i as u32);
            solver.single_sweep();
        }
        solver.sites
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
                                || !self
                                    .cells
                                    .is_active(neighbour.unwrap_entity(), self.current_level)
                        })
                    })
                    .map(move |(entity, _)| Task {
                        entity: *entity,
                        dir: dir_index,
                    })
            })
            .collect();
        tasks
    }

    fn solve(&mut self) {
        let remaining_to_send = 0;
        while self.remaining_to_solve_count.total() > 0 || remaining_to_send > 0 {
            if self.to_solve.is_empty() {
                self.receive_messages();
            }
            while let Some(task) = self.to_solve.pop() {
                self.solve_task(task);
            }
            self.send_all_messages();
        }
    }

    fn receive_messages(&self) {}

    fn send_all_messages(&self) {}

    fn get_outgoing_flux(&mut self, task: &Task) -> PhotonFlux {
        let cell = &self.cells.get(task.entity);
        let site = self.sites.get_mut(task.entity);
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.ionized_hydrogen_fraction);
        let source = site.source_per_direction_bin(&self.directions);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let flux = site.incoming_total_flux[task.dir.0] + source;
        let absorbed_fraction = (-neutral_hydrogen_number_density * sigma * cell.size).exp();
        flux * absorbed_fraction
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = self.get_outgoing_flux(&task);
        let site = self.sites.get_mut(task.entity);
        let outgoing_flux_correction = outgoing_flux - site.outgoing_total_flux[task.dir.0];
        site.outgoing_total_flux[task.dir.0] = outgoing_flux;
        let cell = &self.cells.get(task.entity);
        self.remaining_to_solve_count.reduce(task.dir);
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
                    Neighbour::Local(neighbour_entity) => self.handle_local_neighbour(
                        flux_correction_this_cell,
                        &task,
                        *neighbour_entity,
                    ),
                    Neighbour::Remote(remote) => self.handle_remote_neighbour(remote),
                    Neighbour::Boundary => {}
                }
            }
        }
    }

    fn handle_local_neighbour(
        &mut self,
        incoming_flux_correction: PhotonFlux,
        task: &Task,
        neighbour: Entity,
    ) {
        let (site, is_active) = self
            .sites
            .get_mut_and_active_state(neighbour, self.current_level);
        site.incoming_total_flux[*task.dir] += incoming_flux_correction;
        if is_active {
            let num_remaining = site.num_missing_upwind.reduce(task.dir);
            if num_remaining == 0 {
                self.to_solve.push(Task {
                    dir: task.dir,
                    entity: neighbour,
                })
            }
        }
    }

    fn handle_remote_neighbour(&mut self, _remote: &RemoteNeighbour) {
        todo!()
    }

    pub fn init_counts(&mut self) {
        self.remaining_to_solve_count = CountByDir::new(
            self.directions.len(),
            self.cells.enumerate_active(self.current_level).count(),
        );
        for (entity, cell) in self.cells.enumerate_active(self.current_level) {
            let mut site = self.sites.get_mut(*entity);
            site.num_missing_upwind = CountByDir::new(self.directions.len(), 0);
            for (dir_index, dir) in self.directions.enumerate() {
                for (face, neighbour) in cell.neighbours.iter() {
                    if !neighbour.is_boundary()
                        && face.points_upwind(dir)
                        && self
                            .cells
                            .is_active(neighbour.unwrap_entity(), self.current_level)
                    {
                        site.num_missing_upwind[dir_index] += 1;
                    }
                }
            }
        }
    }

    fn update_chemistry(&mut self) {
        for (entity, cell) in self.cells.enumerate_active(self.current_level) {
            let (level, site) = self.sites.get_mut_with_level(*entity);
            let timestep = level.to_timestep(self.max_timestep);
            let source = site.source_per_direction_bin(&self.directions);
            let flux = site.total_incoming_flux();
            solve_chemistry(
                &mut site.ionized_hydrogen_fraction,
                timestep,
                site.density,
                cell.volume(),
                cell.size,
                source,
                flux,
            );
        }
    }
}

pub fn sweep_system(
    directions: Res<Directions>,
    mut particles: Query<(
        Entity,
        &Cell,
        &Density,
        &mut IonizedHydrogenFraction,
        Option<&Source>,
        &Position,
        &mut TimestepLevel,
    )>,
    timestep: Res<TimestepParameters>,
    sweep_parameters: Res<SweepParameters>,
) {
    let cells = particles
        .iter()
        .map(|(entity, cell, _, _, _, _, _)| (entity, cell.clone()))
        .collect();
    let sites = particles
        .iter()
        .map(
            |(entity, _, density, ionized_hydrogen_fraction, source, _, _)| {
                (
                    entity,
                    Site::new(
                        &directions,
                        **density,
                        **ionized_hydrogen_fraction,
                        source.map(|source| **source).unwrap_or(SourceRate::zero()),
                    ),
                )
            },
        )
        .collect();
    let levels = particles
        .iter()
        .map(|(entity, _, _, _, _, _, level)| (entity, *level))
        .collect();
    let sites = Sweep::run(
        &directions,
        cells,
        sites,
        levels,
        timestep.max_timestep,
        sweep_parameters.num_timestep_levels,
    );
    for (entity, cell, density, mut fraction, _, _, mut level) in particles.iter_mut() {
        let site = sites.get(entity);
        let new_fraction = site.ionized_hydrogen_fraction;
        // let change_timescale =
        //     (**fraction / ((**fraction - new_fraction) / timestep.max_timestep)).abs();
        let change_timescale = cell.volume() * (**density / PROTON_MASS)
            / site.total_incoming_flux()
            * Amount::one_unchecked();
        let desired_timestep = change_timescale * 0.1;
        let mut desired_level = TimestepLevel::from_max_timestep_and_desired_timestep(
            sweep_parameters.num_timestep_levels,
            timestep.max_timestep,
            desired_timestep,
        );
        if desired_level.0 + 1 < level.0 {
            // Never move down more than one level at a time
            desired_level.0 = level.0 - 1;
        }
        level.0 = desired_level.0;
        **fraction = new_fraction;
    }
}

fn initialize_directions_system(mut commands: Commands, parameters: Res<SweepParameters>) {
    let directions: Directions = (&parameters.directions).into();
    commands.insert_resource(directions);
}
