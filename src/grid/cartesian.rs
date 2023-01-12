use bevy::prelude::*;
use bevy::utils::HashMap;

use super::Cell;
use super::Neighbour;
use super::NeighbourKind;
use crate::components;
use crate::components::Position;
use crate::parameters::SimulationBox;
use crate::prelude::Float;
use crate::prelude::LocalParticle;
use crate::units::Density;
use crate::units::Length;
use crate::units::VecLength;

#[derive(PartialEq, Eq, Hash)]
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
            self.x < num_cells.x && self.y < num_cells.y
        }
        #[cfg(not(feature = "2d"))]
        {
            self.x < num_cells.x && self.y < num_cells.y && self.z < num_cells.z
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

    fn to_pos(&self, side_length: VecLength) -> VecLength {
        #[cfg(feature = "2d")]
        {
            VecLength::new(
                side_length.x() * self.x as Float,
                side_length.y() * self.y as Float,
            )
        }

        #[cfg(not(feature = "2d"))]
        {
            VecLength::new(
                side_length.x() * self.x as Float,
                side_length.y() * self.y as Float,
                side_length.z() * self.z as Float,
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

pub fn init_cartesian_grid_system(
    mut commands: Commands,
    box_size: Res<SimulationBox>,
    cell_size: Length,
    density: Density,
) {
    let mut map = HashMap::new();
    let num_cells =
        IntegerPosition::from_position_and_side_length(box_size.side_lengths(), cell_size);
    for integer_pos in num_cells.iter_all_contained() {
        let pos = integer_pos.to_pos(box_size.side_lengths());
        let entity = commands
            .spawn((LocalParticle, Position(pos), components::Density(density)))
            .id();
        map.insert(integer_pos, entity);
    }
    for integer_pos in num_cells.iter_all_contained() {
        let entity = map[&integer_pos];
        let neighbours = integer_pos
            .iter_neighbours()
            .filter_map(|neighbour| {
                if neighbour.contained(&num_cells) {
                    Some(Neighbour {
                        entity: map[&integer_pos],
                        kind: NeighbourKind::Local,
                    })
                } else {
                    None
                }
            })
            .collect();
        commands.entity(entity).insert(Cell { neighbours });
    }
}
