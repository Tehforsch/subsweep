use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Resource;

use crate::units::Dimensionless;
use crate::units::MVec;
use crate::units::VecDimensionless;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct DirectionIndex(usize);

#[derive(Deref, DerefMut)]
pub struct Direction(VecDimensionless);

#[derive(Resource)]
pub struct Directions {
    directions: Vec<Direction>,
}

impl Directions {
    pub fn from_num(num: usize) -> Self {
        match num {
            1 => Self {
                directions: vec![Direction(MVec::X * Dimensionless::dimensionless(1.0))],
            },
            _ => todo!(),
        }
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (DirectionIndex, &Direction)> {
        self.directions
            .iter()
            .enumerate()
            .map(|(i, dir)| (DirectionIndex(i), dir))
    }
}
