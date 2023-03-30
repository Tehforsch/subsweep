use bevy::prelude::*;
use bevy::utils::HashMap;
use derive_custom::raxiom_parameters;

use super::cell::Face;
use super::cell::FaceArea;
use super::Cell;
use super::NeighbourType;
use super::RemoteNeighbour;
use crate::communication::Rank;
use crate::components::Position;
use crate::config::NUM_DIMENSIONS;
use crate::parameters::SimulationBox;
use crate::particle::HaloParticle;
use crate::particle::ParticleId;
use crate::prelude::Float;
use crate::prelude::LocalParticle;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::units::Length;
use crate::units::VecLength;
use crate::units::Volume;

#[raxiom_parameters]
#[derive(Copy)]
#[serde(untagged)]
pub enum NumCellsSpec {
    CellSize(Length),
    NumCellsX(usize),
}

impl NumCellsSpec {
    fn num_cells(&self, box_size: &SimulationBox) -> IntegerPosition {
        match self {
            NumCellsSpec::CellSize(cell_size) => {
                IntegerPosition::from_position_and_side_length(box_size.side_lengths(), *cell_size)
            }
            NumCellsSpec::NumCellsX(num_cells_x) => IntegerPosition {
                x: *num_cells_x as i32,
                y: 1,
                #[cfg(not(feature = "2d"))]
                z: 1,
            },
        }
    }

    fn cell_size(&self, box_size: &SimulationBox) -> Length {
        match self {
            NumCellsSpec::CellSize(cell_size) => *cell_size,
            NumCellsSpec::NumCellsX(num_cells_x) => {
                box_size.side_lengths().x() / *num_cells_x as f64
            }
        }
    }

    fn face_area(&self, box_size: &SimulationBox) -> FaceArea {
        match self {
            NumCellsSpec::CellSize(cell_size) => {
                #[cfg(feature = "2d")]
                {
                    *cell_size
                }
                #[cfg(not(feature = "2d"))]
                {
                    cell_size.powi::<2>()
                }
            }
            NumCellsSpec::NumCellsX(_) => {
                #[cfg(feature = "2d")]
                {
                    box_size.side_lengths().y()
                }
                #[cfg(not(feature = "2d"))]
                {
                    box_size.side_lengths().y() * box_size.side_lengths().z()
                }
            }
        }
    }

    fn volume(&self, box_size: &SimulationBox) -> Volume {
        match self {
            NumCellsSpec::CellSize(cell_size) => cell_size.powi::<{ NUM_DIMENSIONS as i32 }>(),
            NumCellsSpec::NumCellsX(num_cells_x) => box_size.volume() / *num_cells_x as f64,
        }
    }
}

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

    fn to_pos(self, side_length: VecLength, num_particles: &Self) -> VecLength {
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
    ids: HashMap<IntegerPosition, ParticleId>,
    box_size: SimulationBox,
    resolution: NumCellsSpec,
    rank_function: Box<dyn Fn(VecLength) -> Rank>,
    rank: Rank,
}

impl GridConstructor {
    fn construct(
        commands: Commands,
        box_size: SimulationBox,
        cell_size: NumCellsSpec,
        rank_function: Box<dyn Fn(VecLength) -> Rank>,
        rank: Rank,
    ) {
        let mut constructor = Self {
            cells: HashMap::default(),
            ids: HashMap::default(),
            box_size,
            resolution: cell_size,
            rank_function,
            rank,
        };
        for (i, integer_pos) in constructor
            .get_all_integer_positions()
            .into_iter()
            .enumerate()
        {
            constructor.ids.insert(integer_pos, ParticleId(i as u64));
        }
        constructor.construct_neighbours();
        constructor.spawn_local_cells(commands);
    }

    fn num_cells(&self) -> IntegerPosition {
        self.resolution.num_cells(&self.box_size)
    }

    fn volume(&self) -> Volume {
        self.resolution.volume(&self.box_size)
    }

    fn face_area(&self) -> FaceArea {
        self.resolution.face_area(&self.box_size)
    }

    fn cell_size(&self) -> Length {
        self.resolution.cell_size(&self.box_size)
    }

    fn construct_neighbours(&mut self) {
        for integer_pos in self.get_all_integer_positions() {
            let pos = self.to_pos(integer_pos);
            let rank = self.get_rank(integer_pos);
            let neighbours = integer_pos
                .iter_neighbours()
                .map(|neighbour| {
                    let neighbour_pos = self.to_pos(neighbour);
                    let neighbour_rank = self.get_rank(neighbour);
                    let face = Face {
                        area: self.face_area(),
                        normal: (neighbour_pos - pos).normalize(),
                    };
                    if neighbour.contained(&self.num_cells()) {
                        if rank == neighbour_rank {
                            (face, NeighbourType::Local(self.ids[&neighbour]))
                        } else {
                            (
                                face,
                                NeighbourType::Remote(RemoteNeighbour {
                                    id: self.ids[&neighbour],
                                    rank: self.get_rank(neighbour),
                                }),
                            )
                        }
                    } else {
                        (face, NeighbourType::Boundary)
                    }
                })
                .collect();
            let cell = Cell {
                neighbours,
                size: self.cell_size(),
                volume: self.volume(),
            };
            self.cells.insert(integer_pos, cell);
        }
    }

    fn get_all_integer_positions(&self) -> Vec<IntegerPosition> {
        self.num_cells().iter_all_contained().collect()
    }

    fn to_pos(&self, integer_pos: IntegerPosition) -> VecLength {
        integer_pos.to_pos(self.box_size.side_lengths(), &self.num_cells())
    }

    fn get_rank(&self, pos: IntegerPosition) -> Rank {
        let pos = self.to_pos(pos);
        (self.rank_function)(pos)
    }

    fn spawn_local_cells(&mut self, mut commands: Commands) {
        let drained_cells: Vec<_> = self.cells.drain().collect();
        for (integer_pos, cell) in drained_cells {
            let particle_id = self.ids[&integer_pos];
            let pos = self.to_pos(integer_pos);
            let rank = self.get_rank(integer_pos);
            if rank == self.rank {
                commands.spawn((LocalParticle, Position(pos), cell, particle_id));
            } else if cell.neighbours.iter().any(|(_, neighbour)| {
                if let NeighbourType::Remote(neighbour) = neighbour {
                    neighbour.rank == self.rank
                } else {
                    false
                }
            }) {
                commands.spawn((HaloParticle { rank }, Position(pos), cell, particle_id));
            }
        }
    }
}

pub fn init_cartesian_grid_system(
    commands: Commands,
    box_size: Res<SimulationBox>,
    cell_size: NumCellsSpec,
    world_size: Res<WorldSize>,
    world_rank: Res<WorldRank>,
) {
    let cloned_box_size = box_size.clone();
    let cloned_world_size = *world_size;
    let rank_function = move |pos: VecLength| {
        ((pos.x() / cloned_box_size.side_lengths().x()) * cloned_world_size.0 as f64)
            .floor()
            .round() as Rank
    };
    GridConstructor::construct(
        commands,
        box_size.clone(),
        cell_size,
        Box::new(rank_function),
        **world_rank,
    );
}
