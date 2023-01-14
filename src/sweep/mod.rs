pub mod components;
mod count_by_dir;
mod direction;
mod parameters;
mod site;
mod task;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
pub use parameters::SweepParameters;

use self::components::HydrogenAbundance;
use self::components::Source;
use self::count_by_dir::CountByDir;
use self::direction::Direction;
use self::direction::Directions;
use self::site::Site;
use self::task::Task;
use crate::components::Density;
use crate::components::Position;
use crate::grid::Cell;
use crate::grid::Neighbour;
use crate::grid::RemoteNeighbour;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::PhotonFlux;
use crate::units::SourceRate;
use crate::units::VecDimensionless;
use crate::units::PROTON_MASS;
type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;

type CellQuery<'w, 's> = Particles<'w, 's, (Entity, &'static Cell, &'static Position)>;
type SiteQuery<'w, 's> = Particles<
    'w,
    's,
    (
        &'static mut Site,
        &'static Density,
        &'static HydrogenAbundance,
    ),
>;
type SourceQuery<'w, 's> = Particles<'w, 's, &'static mut Source>;

#[derive(Named)]
pub struct SweepPlugin;

impl RaxiomPlugin for SweepPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            initialize_sites_system,
        )
        .add_required_component::<HydrogenAbundance>()
        .add_required_component::<Source>()
        .add_system(init_counts_system.before(sweep_system))
        .add_system(sweep_system)
        .add_parameter_type::<SweepParameters>();
    }
}

struct Sweep<'w, 's> {
    directions: Directions,
    cells: CellQuery<'w, 's>,
    sites: SiteQuery<'w, 's>,
    sources: SourceQuery<'w, 's>,
    to_solve: PriorityQueue<Task>,
    remaining_to_solve_count: CountByDir,
}

impl<'w, 's> Sweep<'w, 's> {
    fn run(
        parameters: &SweepParameters,
        cells: CellQuery<'w, 's>,
        sites: SiteQuery,
        sources: SourceQuery,
    ) {
        let directions: Directions = (&parameters.directions).into();
        let remaining_to_solve = CountByDir::new(directions.len(), cells.iter().count());
        let mut solver = Sweep {
            cells,
            sites,
            sources,
            to_solve: PriorityQueue::new(),
            directions,
            remaining_to_solve_count: remaining_to_solve,
        };
        solver.add_initial_tasks();
        solver.solve();
    }

    fn add_initial_tasks(&mut self) {
        let tasks = self
            .directions
            .enumerate()
            .flat_map(|(dir_index, dir)| {
                self.cells
                    .iter()
                    .filter(|entry| {
                        let cell1 = entry.1;
                        // Importantly, the !face_points_upwind cannot
                        // be changed to face_points_downwind, because
                        // we need to be inclusive of all faces, even
                        // those that have zero dot product with the
                        // face normal.
                        cell1
                            .neighbours
                            .iter()
                            .all(|(face, _)| !face_points_upwind(&face.normal, dir))
                    })
                    .map(move |(entity, _, _)| Task {
                        entity,
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
        let cell = self.cells.get_component::<Cell>(task.entity).unwrap();
        let density = **self.sites.get_component::<Density>(task.entity).unwrap();
        let hydrogen_abundance = **self
            .sites
            .get_component::<HydrogenAbundance>(task.entity)
            .unwrap();
        let hydrogen_number_density = density / PROTON_MASS * hydrogen_abundance;
        let source = match self.sources.get_component::<Source>(task.entity) {
            Ok(source) => **source,
            Err(_) => SourceRate::zero(),
        };
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        task.flux * (-hydrogen_number_density * sigma * cell.size).exp() + source
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = self.solve_eq(&task);
        let cell = self.cells.get_component::<Cell>(task.entity).unwrap();
        self.remaining_to_solve_count.reduce(task.dir);
        // This is very inefficient, let's see if this ever becomes a bottleneck
        let neighbours = cell.neighbours.clone();
        for (face, neighbour) in neighbours.iter() {
            if face_points_downwind(&face.normal, &self.directions[task.dir]) {
                match neighbour {
                    Neighbour::Local(neighbour_entity) => {
                        self.handle_local_neighbour(outgoing_flux, &task, *neighbour_entity)
                    }
                    Neighbour::Remote(remote) => self.handle_remote_neighbour(remote),
                }
            }
        }
    }

    fn handle_local_neighbour(
        &mut self,
        outgoing_flux: PhotonFlux,
        task: &Task,
        neighbour: Entity,
    ) {
        let mut site = self.sites.get_component_mut::<Site>(neighbour).unwrap();
        site.num_missing_upwind.reduce(task.dir);
        if site.num_missing_upwind[task.dir] == 0 {
            self.to_solve.push(Task {
                dir: task.dir,
                entity: neighbour,
                flux: outgoing_flux,
            })
        }
    }

    fn handle_remote_neighbour(&mut self, _remote: &RemoteNeighbour) {
        todo!()
    }
}

fn init_counts_system(cells: CellQuery, mut sites: SiteQuery, parameters: Res<SweepParameters>) {
    for (entity, cell, _) in cells.iter() {
        let (mut site, _, _) = sites.get_mut(entity).unwrap();
        site.num_missing_upwind = CountByDir::new(parameters.directions.len(), 0);
        let directions: Directions = (&parameters.directions).into();
        for (dir_index, dir) in directions.enumerate() {
            for (face, _) in cell.neighbours.iter() {
                if face_points_upwind(&face.normal, dir) {
                    site.num_missing_upwind[dir_index] += 1;
                }
            }
        }
    }
}

fn sweep_system(
    parameters: Res<SweepParameters>,
    cells: CellQuery,
    sites: SiteQuery,
    sources: SourceQuery,
) {
    Sweep::run(&parameters, cells, sites, sources);
}

fn initialize_sites_system(mut commands: Commands, cells: CellQuery) {
    for (entity, _, _) in cells.iter() {
        commands.entity(entity).insert(Site {
            num_missing_upwind: CountByDir::empty(),
        });
    }
}

pub(super) fn face_points_upwind(normal: &VecDimensionless, dir: &Direction) -> bool {
    normal.dot(**dir).is_negative()
}

pub(super) fn face_points_downwind(normal: &VecDimensionless, dir: &Direction) -> bool {
    normal.dot(**dir).is_positive()
}
