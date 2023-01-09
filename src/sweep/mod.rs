mod direction;
mod parameters;
mod site;
mod task;
mod task_counter;

use bevy::prelude::*;
pub use parameters::SweepParameters;

use self::direction::Direction;
use self::direction::Directions;
use self::task::Task;
use self::task_counter::TaskCounter;
use crate::components::Position;
use crate::grid::Cell;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::Flux;
use crate::units::VecLength;

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;

type QueryType<'w, 's> = Query<'w, 's, (Entity, &'static Cell, &'static Position)>;

#[derive(Named)]
pub struct SweepPlugin;

impl RaxiomPlugin for SweepPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system(sweep_system)
            .add_parameter_type::<SweepParameters>();
    }
}

struct Sweep<'w, 's> {
    pub directions: Directions,
    pub cells: QueryType<'w, 's>,
    pub to_solve: PriorityQueue<Task>,
    pub remaining_to_solve_count: TaskCounter,
}

impl<'w, 's> Sweep<'w, 's> {
    fn run(parameters: &SweepParameters, cells: QueryType<'w, 's>) {
        let directions = Directions::from_num(parameters.num_directions);
        let remaining_to_solve = TaskCounter::new(&directions, cells.iter().count());
        let mut solver = Sweep {
            cells,
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
            if self.to_solve.len() == 0 {
                self.receive_messages();
            }
            while let Some(task) = self.to_solve.pop() {
                self.solve_task(task);
            }
        }
    }

    fn receive_messages(&self) {}

    fn solve_task(&mut self, task: Task) {
        self.remaining_to_solve_count.reduce(task.dir);
    }
}

fn sweep_system(parameters: Res<SweepParameters>, cells: QueryType) {
    Sweep::run(&parameters, cells);
}

/// Returns whether pos1 is upwind of pos2 along dir.
fn is_upwind(pos1: &VecLength, pos2: &VecLength, dir: &Direction) -> bool {
    let dist = *pos1 - *pos2;
    dist.dot(**dir).is_negative()
}
