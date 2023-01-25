mod active_list;
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
use crate::units::Dimensionless;
use crate::units::PhotonFlux;
use crate::units::SourceRate;
use crate::units::Time;
use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
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
        let remaining_to_solve = CountByDir::new(directions.len(), cells.iter().count());
        let mut solver = Sweep {
            cells: Cells::new(cells, &levels),
            sites: Sites::new(sites, &levels),
            to_solve: PriorityQueue::new(),
            directions: directions.clone(),
            remaining_to_solve_count: remaining_to_solve,
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
        self.init_counts();
        self.add_initial_tasks();
        self.solve();
        self.update_chemistry();
    }

    fn add_initial_tasks(&mut self) {
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
                            !face.points_upwind(dir) || neighbour.is_boundary()
                        })
                    })
                    .map(move |(entity, _)| Task {
                        entity: *entity,
                        dir: dir_index,
                        flux: PhotonFlux::zero(),
                    })
            })
            .collect();
        self.to_solve = tasks;
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

    fn solve_eq(&mut self, task: &Task) -> PhotonFlux {
        let cell = &self.cells[task.entity];
        let site = self.sites.get_mut(task.entity).unwrap();
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.ionized_hydrogen_fraction);
        let source = site.source / self.directions.len() as f64;
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let flux = task.flux + source;
        let absorbed_fraction = 1.0 - (-neutral_hydrogen_number_density * sigma * cell.size).exp();
        flux * (1.0 - absorbed_fraction)
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = self.solve_eq(&task);
        let cell = &self.cells[task.entity];
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
                let flux_this_cell = outgoing_flux * (effective_area / total_effective_area);
                match neighbour {
                    Neighbour::Local(neighbour_entity) => {
                        self.handle_local_neighbour(flux_this_cell, &task, *neighbour_entity)
                    }
                    Neighbour::Remote(remote) => self.handle_remote_neighbour(remote),
                    Neighbour::Boundary => {}
                }
            }
        }
    }

    fn handle_local_neighbour(
        &mut self,
        incoming_flux: PhotonFlux,
        task: &Task,
        neighbour: Entity,
    ) {
        let site = self.sites.get_mut(neighbour).unwrap();
        let num_remaining = site.num_missing_upwind.reduce(task.dir);
        site.flux[*task.dir] += incoming_flux;
        if num_remaining == 0 {
            self.to_solve.push(Task {
                dir: task.dir,
                entity: neighbour,
                flux: site.flux[*task.dir],
            })
        }
    }

    fn handle_remote_neighbour(&mut self, _remote: &RemoteNeighbour) {
        todo!()
    }

    pub fn init_counts(&mut self) {
        for (entity, cell) in self.cells.enumerate_active(self.current_level) {
            let mut site = self.sites.get_mut(*entity).unwrap();
            site.num_missing_upwind = CountByDir::new(self.directions.len(), 0);
            for (dir_index, dir) in self.directions.enumerate() {
                for (face, neighbour) in cell.neighbours.iter() {
                    if !neighbour.is_boundary() && face.points_upwind(dir) {
                        site.num_missing_upwind[dir_index] += 1;
                    }
                }
            }
        }
    }

    fn update_chemistry(&mut self) {
        for (entity, cell) in self.cells.enumerate_active(self.current_level) {
            let mut site = self.sites.get_mut(*entity).unwrap();
            let hydrogen_number_density = site.density / PROTON_MASS;
            let num_hydrogen_atoms = hydrogen_number_density * cell.volume();
            let recombination_rate = CASE_B_RECOMBINATION_RATE_HYDROGEN
                * (hydrogen_number_density * site.ionized_hydrogen_fraction).powi::<2>();
            let num_recombined_hydrogen_atoms =
                (recombination_rate * self.max_timestep * cell.volume()).to_amount();
            let neutral_hydrogen_number_density =
                site.density / PROTON_MASS * (1.0 - site.ionized_hydrogen_fraction);
            let source = site.source / self.directions.len() as f64;
            let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
            let flux = site.total_flux() + source;
            let absorbed_fraction =
                1.0 - (-neutral_hydrogen_number_density * sigma * cell.size).exp();
            let num_newly_ionized_hydrogen_atoms = (absorbed_fraction * flux) * self.max_timestep;
            site.ionized_hydrogen_fraction += (num_newly_ionized_hydrogen_atoms
                - num_recombined_hydrogen_atoms)
                / num_hydrogen_atoms.to_amount();
            site.ionized_hydrogen_fraction = site.ionized_hydrogen_fraction.clamp(
                Dimensionless::dimensionless(0.0),
                Dimensionless::dimensionless(1.0),
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
    )>,
    timestep: Res<TimestepParameters>,
    sweep_parameters: Res<SweepParameters>,
    simulation_box: Res<SimulationBox>,
) {
    let cells = particles
        .iter()
        .map(|(entity, cell, _, _, _, _)| (entity, cell.clone()))
        .collect();
    let sites = particles
        .iter()
        .map(
            |(entity, _, density, ionized_hydrogen_fraction, source, _)| {
                (
                    entity,
                    Site {
                        density: **density,
                        ionized_hydrogen_fraction: **ionized_hydrogen_fraction,
                        source: source.map(|source| **source).unwrap_or(SourceRate::zero()),
                        num_missing_upwind: CountByDir::empty(),
                        flux: directions.enumerate().map(|_| PhotonFlux::zero()).collect(),
                    },
                )
            },
        )
        .collect();
    let levels = particles
        .iter()
        .map(|(entity, _, _, _, _, pos)| {
            if pos.x() < simulation_box.center().x() {
                (entity, TimestepLevel(0))
            } else {
                (entity, TimestepLevel(0))
            }
        })
        .collect();
    let sites = Sweep::run(
        &directions,
        cells,
        sites,
        levels,
        timestep.max_timestep,
        sweep_parameters.num_timestep_levels,
    );
    for (entity, _, _, mut fraction, _, _) in particles.iter_mut() {
        **fraction = sites[entity].ionized_hydrogen_fraction;
    }
}

fn initialize_directions_system(mut commands: Commands, parameters: Res<SweepParameters>) {
    let directions: Directions = (&parameters.directions).into();
    commands.insert_resource(directions);
}
