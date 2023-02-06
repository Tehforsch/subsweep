use bevy::prelude::*;
use bevy::utils::HashMap;

use super::cell::Face;
use super::cell::FaceArea;
use super::Cell;
use super::Neighbour;
use crate::communication::Rank;
use crate::components::Position;
use crate::parameters::SimulationBox;
use crate::prelude::Float;
use crate::prelude::LocalParticle;
use crate::prelude::WorldSize;
use crate::units::Length;
use crate::units::VecLength;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct IntegerPosition {
    x: i32,
    y: i32,
    #[cfg(not(feature = "2d"))]
    z: i32,
}

impl IntegerPosition {
    fn contained(&self, num_cells: &IntegerPosition) -> bool {
        #[cfg(feature = "2d")]
        {
            0 <= self.x && 0 <= self.y && self.x < num_cells.x && self.y < num_cells.y
        }
        #[cfg(not(feature = "2d"))]
        {
            0 <= self.x
                && 0 <= self.y
                && 0 <= self.z
                && self.x < num_cells.x
                && self.y < num_cells.y
                && self.z < num_cells.z
        }
    }

    fn from_position_and_side_length(pos: VecLength, side_length: Length) -> IntegerPosition {
        let float = (pos / side_length).value();
        #[cfg(feature = "2d")]
        {
            Self {
                x: float.x.floor() as i32,
                y: float.y.floor() as i32,
            }
        }
        #[cfg(not(feature = "2d"))]
        {
            Self {
                x: float.x.floor() as i32,
                y: float.y.floor() as i32,
                z: float.z.floor() as i32,
            }
        }
    }

    fn to_pos(&self, side_length: VecLength, num_particles: &Self) -> VecLength {
        #[cfg(feature = "2d")]
        {
            VecLength::new(
                side_length.x() * self.x as Float / num_particles.x as Float,
                side_length.y() * self.y as Float / num_particles.y as Float,
            )
        }

        #[cfg(not(feature = "2d"))]
        {
            VecLength::new(
                side_length.x() * self.x as Float / num_particles.x as Float,
                side_length.y() * self.y as Float / num_particles.y as Float,
                side_length.z() * self.z as Float / num_particles.z as Float,
            )
        }
    }

    fn iter_all_contained(&self) -> impl Iterator<Item = IntegerPosition> + '_ {
        #[cfg(feature = "2d")]
        {
            (0..self.x).flat_map(move |x| (0..self.y).map(move |y| Self { x, y }))
        }
        #[cfg(not(feature = "2d"))]
        {
            (0..self.x).flat_map(move |x| {
                (0..self.y).flat_map(move |y| (0..self.z).map(move |z| Self { x, y, z }))
            })
        }
    }

    fn iter_neighbours(&self) -> impl Iterator<Item = IntegerPosition> {
        #[cfg(feature = "2d")]
        {
            [
                (self.x - 1, self.y),
                (self.x + 1, self.y),
                (self.x, self.y - 1),
                (self.x, self.y + 1),
            ]
            .into_iter()
            .map(move |(x, y)| Self { x, y })
        }
        #[cfg(not(feature = "2d"))]
        {
            [
                (self.x - 1, self.y, self.z),
                (self.x + 1, self.y, self.z),
                (self.x, self.y - 1, self.z),
                (self.x, self.y + 1, self.z),
                (self.x, self.y, self.z - 1),
                (self.x, self.y, self.z + 1),
            ]
            .into_iter()
            .map(move |(x, y, z)| Self { x, y, z })
        }
    }
}

struct GridConstructor {
    cells: HashMap<IntegerPosition, Cell>,
    entities: HashMap<IntegerPosition, Entity>,
    box_size: SimulationBox,
    cell_size: Length,
    num_cells: IntegerPosition,
    rank_function: Box<dyn Fn(VecLength) -> Rank>,
}

impl GridConstructor {
    fn construct(
        mut commands: Commands,
        box_size: SimulationBox,
        cell_size: Length,
        rank_function: Box<dyn Fn(VecLength) -> Rank>,
    ) {
        let num_cells =
            IntegerPosition::from_position_and_side_length(box_size.side_lengths(), cell_size);
        let mut constructor = Self {
            cells: HashMap::default(),
            entities: HashMap::default(),
            box_size,
            cell_size,
            num_cells,
            rank_function,
        };
        for integer_pos in constructor.get_all_integer_positions() {
            let entity = commands.spawn(LocalParticle).id();
            constructor.entities.insert(integer_pos, entity);
        }
        constructor.construct_neighbours();
        constructor.spawn_local_cells(commands);
    }

    fn construct_neighbours(&mut self) {
        for integer_pos in self.get_all_integer_positions() {
            let pos = self.to_pos(integer_pos);
            let entity = self.entities[&integer_pos];
            let neighbours = integer_pos
                .iter_neighbours()
                .map(|neighbour| {
                    let neighbour_pos = self.to_pos(neighbour);
                    let face = Face {
                        area: get_area(self.cell_size),
                        normal: (neighbour_pos - pos).normalize(),
                    };
                    if neighbour.contained(&self.num_cells) {
                        (face, Neighbour::Local(self.entities[&neighbour]))
                    } else {
                        (face, Neighbour::Boundary)
                    }
                })
                .collect();
            let cell = Cell {
                neighbours,
                size: self.cell_size,
            };
            self.cells.insert(integer_pos, cell);
        }
    }

    fn get_all_integer_positions(&self) -> Vec<IntegerPosition> {
        self.num_cells.iter_all_contained().collect()
    }

    fn to_pos(&self, integer_pos: IntegerPosition) -> VecLength {
        integer_pos.to_pos(self.box_size.side_lengths(), &self.num_cells)
    }

    fn spawn_local_cells(&mut self, mut commands: Commands) {
        let drained_cells: Vec<_> = self.cells.drain().collect();
        for (integer_pos, cell) in drained_cells {
            let entity = self.entities[&integer_pos];
            let pos = self.to_pos(integer_pos);
            commands.entity(entity).insert((Position(pos), cell));
        }
    }
}

pub fn init_cartesian_grid_system(
    commands: Commands,
    box_size: Res<SimulationBox>,
    cell_size: Length,
    world_size: Res<WorldSize>,
) {
    let cloned_box_size = box_size.clone();
    let cloned_world_size = world_size.clone();
    let rank_function = move |pos: VecLength| {
        ((pos.x() / cloned_box_size.side_lengths().x()) * cloned_world_size.0 as f64).round()
            as Rank
    };
    GridConstructor::construct(
        commands,
        box_size.clone(),
        cell_size,
        Box::new(rank_function),
    );
}

fn get_area(cell_size: Length) -> FaceArea {
    #[cfg(feature = "2d")]
    {
        cell_size
    }
    #[cfg(not(feature = "2d"))]
    {
        cell_size * cell_size
    }
}
