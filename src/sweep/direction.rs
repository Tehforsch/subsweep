use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Resource;
use serde::Deserialize;
use serde::Serialize;

use super::parameters::DirectionsSpecification;
use crate::units::Dimensionless;
use crate::units::MVec;
use crate::units::VecDimensionless;

#[derive(Deref, DerefMut, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct DirectionIndex(usize);

#[derive(Deref, DerefMut, Deserialize, Serialize, Clone)]
pub struct Direction(pub VecDimensionless);

#[derive(Resource)]
pub struct Directions {
    directions: Vec<Direction>,
}

impl Directions {
    fn from_num(num: usize) -> Self {
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

    pub fn len(&self) -> usize {
        self.directions.len()
    }
}

impl std::ops::Index<DirectionIndex> for Directions {
    type Output = Direction;

    fn index(&self, index: DirectionIndex) -> &Self::Output {
        &self.directions[index.0]
    }
}

impl From<&DirectionsSpecification> for Directions {
    fn from(value: &DirectionsSpecification) -> Self {
        match value {
            DirectionsSpecification::Num(num) => Self::from_num(*num),
            DirectionsSpecification::Explicit(ref directions) => Self {
                directions: directions
                    .iter()
                    .map(|dir| Direction(dir.clone().normalize()))
                    .collect(),
            },
        }
    }
}
