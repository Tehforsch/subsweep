mod count_by_dir;
mod direction;
mod parameters;
mod site;
mod task;

use bevy::prelude::*;
pub use parameters::SweepParameters;

use self::count_by_dir::CountByDir;
use self::direction::Direction;
use self::direction::Directions;
use self::site::Site;
use self::task::Task;
use crate::components::Density;
use crate::components::Position;
use crate::grid::Cell;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::Flux;
use crate::units::VecLength;

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;

type CellQuery<'w, 's> = Particles<'w, 's, (Entity, &'static Cell, &'static Position)>;
type SiteQuery<'w, 's> = Particles<'w, 's, (&'static mut Site, &'static Density)>;

#[derive(Named)]
pub struct SweepPlugin;

impl RaxiomPlugin for SweepPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            initialize_sites_system,
        )
        .add_system(init_counts_system.before(sweep_system))
        .add_system(sweep_system)
        .add_parameter_type::<SweepParameters>();
    }
}

struct Sweep<'w, 's> {
    pub directions: Directions,
    pub cells: CellQuery<'w, 's>,
    pub sites: SiteQuery<'w, 's>,
    pub to_solve: PriorityQueue<Task>,
    pub remaining_to_solve_count: CountByDir,
}

impl<'w, 's> Sweep<'w, 's> {
    fn run(parameters: &SweepParameters, cells: CellQuery<'w, 's>, sites: SiteQuery) {
        let directions = Directions::from_num(parameters.num_directions);
        let remaining_to_solve = CountByDir::new(parameters.num_directions, cells.iter().count());
        let mut solver = Sweep {
            cells,
            sites,
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
                        let pos1 = entry.2;
                        let has_no_upwind_neighbours = cell1
                            .neighbours
                            .iter()
                            .filter(|neighbour| {
                                let pos2 = self
                                    .cells
                                    .get_component::<Position>(neighbour.entity)
                                    .unwrap();
                                is_upwind(pos2, pos1, dir)
                            })
                            .count()
                            == 0;
                        has_no_upwind_neighbours
                    })
                    .map(move |(entity, _, _)| Task {
                        entity,
                        dir: dir_index,
                        flux: Flux::zero(),
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
        }
    }

    fn receive_messages(&self) {}

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = solve_eq(&task);
        let pos1 = self.cells.get_component::<Position>(task.entity).unwrap();
        self.remaining_to_solve_count.reduce(task.dir);
        for neighbour in self
            .cells
            .get_component::<Cell>(task.entity)
            .unwrap()
            .neighbours
            .iter()
        {
            let (mut site, _) = self.sites.get_mut(neighbour.entity).unwrap();
            let pos2 = self
                .cells
                .get_component::<Position>(neighbour.entity)
                .unwrap();
            if is_upwind(pos1, pos2, &self.directions[task.dir]) {
                site.num_missing_upwind.reduce(task.dir);
                if site.num_missing_upwind[task.dir] == 0 {
                    self.to_solve.push(Task {
                        dir: task.dir,
                        entity: neighbour.entity,
                        flux: outgoing_flux,
                    })
                }
            }
        }
    }
}

fn solve_eq(_task: &Task) -> Flux {
    Flux::zero()
}

fn init_counts_system(cells: CellQuery, mut sites: SiteQuery, parameters: Res<SweepParameters>) {
    for (entity, cell, pos1) in cells.iter() {
        let (mut site, _) = sites.get_mut(entity).unwrap();
        site.num_missing_upwind = CountByDir::new(parameters.num_directions, 0);
        for (dir_index, dir) in Directions::from_num(parameters.num_directions).enumerate() {
            for neighbour in cell.neighbours.iter() {
                let pos2 = cells.get_component::<Position>(neighbour.entity).unwrap();
                if is_upwind(pos2, pos1, dir) {
                    site.num_missing_upwind[dir_index] += 1;
                }
            }
        }
    }
}
fn sweep_system(parameters: Res<SweepParameters>, cells: CellQuery, sites: SiteQuery) {
    Sweep::run(&parameters, cells, sites);
}

fn initialize_sites_system(mut commands: Commands, cells: CellQuery) {
    for (entity, _, _) in cells.iter() {
        commands.entity(entity).insert(Site {
            num_missing_upwind: CountByDir::empty(),
        });
    }
}

/// Returns whether pos1 is upwind of pos2 along dir.
fn is_upwind(pos1: &VecLength, pos2: &VecLength, dir: &Direction) -> bool {
    let dist = *pos1 - *pos2;
    dist.dot(**dir).is_negative()
}
