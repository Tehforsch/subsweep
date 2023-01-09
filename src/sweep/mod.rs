mod direction;
mod parameters;
mod site;
mod task;

use bevy::prelude::*;
pub use parameters::SweepParameters;

use self::direction::Direction;
use self::direction::Directions;
use self::task::Task;
use crate::components::Position;
use crate::grid::Cell;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::Flux;
use crate::units::VecLength;

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;

type QueryType<'w, 's> = Query<'w, 's, (Entity, &'static Cell, &'static Position)>;

#[derive(Named)]
struct SweepPlugin {}

impl RaxiomPlugin for SweepPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system(sweep_system);
    }
}

struct Solver<'w, 's> {
    pub directions: Directions,
    pub cells: QueryType<'w, 's>,
    pub to_solve: PriorityQueue<Task>,
}

impl<'w, 's> Solver<'w, 's> {
    fn new(parameters: &SweepParameters, cells: QueryType<'w, 's>) -> Self {
        let mut solver = Solver {
            cells,
            to_solve: PriorityQueue::new(),
            directions: Directions::from_num(parameters.num_directions),
        };
        solver.add_initial_tasks();
        solver
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
                        let has_upwind_neighbours = cell1
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
                        !has_upwind_neighbours
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
}

fn sweep_system(
    parameters: Res<SweepParameters>,
    cells: QueryType,
) {
    let solver = Solver::new(&parameters, cells);
}

/// Returns whether pos1 is upwind of pos2 along dir.
fn is_upwind(pos1: &VecLength, pos2: &VecLength, dir: &Direction) -> bool {
    let dist = *pos1 - *pos2;
    dist.dot(**dir).is_negative()
}
