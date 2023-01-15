#[cfg(not(feature = "2d"))]
mod healpix;

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

#[derive(Deref, DerefMut, Deserialize, Serialize, Clone, Debug)]
pub struct Direction(pub VecDimensionless);

#[derive(Resource)]
pub struct Directions {
    directions: Vec<Direction>,
}

impl Directions {
    #[cfg(feature = "2d")]
    fn from_num(num: usize) -> Self {
        use std::f64::consts::PI;

        Self {
            directions: (0..num)
                .map(|i| {
                    let fraction = 0.125 + (i as f64) / (num as f64);
                    let x = (fraction * 2.0 * PI).cos();
                    let y = (fraction * 2.0 * PI).sin();
                    Direction(MVec::new(x, y) * Dimensionless::dimensionless(1.0))
                })
                .collect(),
        }
    }

    #[cfg(not(feature = "2d"))]
    fn from_num(num: usize) -> Self {
        let bins: &[&[f64; 3]] = match num {
            1 => &[&[1.0, 0.0, 0.0]],
            84 => &healpix::DIRECTION_BINS_84,
            _ => unimplemented!(),
        };
        Self {
            directions: bins
                .iter()
                .map(|&[x, y, z]| {
                    Direction(MVec::new(*x, *y, *z) * Dimensionless::dimensionless(1.0))
                })
                .collect(),
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
