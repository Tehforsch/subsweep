#[cfg(not(feature = "2d"))]
mod healpix;

use std::f64::consts::PI;

use bevy_ecs::prelude::NonSendMut;
use bevy_ecs::prelude::ResMut;
use bevy_ecs::prelude::Resource;
use derive_more::Deref;
use derive_more::DerefMut;
use mpi::traits::Equivalence;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;
use serde::Serialize;

use super::parameters::DirectionsSpecification;
use super::Sweep;
use crate::chemistry::hydrogen_only::HydrogenOnly;
use crate::prelude::Simulation;
use crate::units::Dimensionless;
use crate::units::MVec;
use crate::units::VecDimensionless;

#[derive(
    Deref, DerefMut, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Hash, Equivalence,
)]
pub struct DirectionIndex(pub usize);

#[derive(Deref, DerefMut, Deserialize, Serialize, Clone, Debug)]
pub struct Direction(pub VecDimensionless);

#[derive(Resource, Clone)]
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
        let bins: &[[f64; 3]] = match num {
            1 => &[[1.0, 0.0, 0.0]],
            16 => &healpix::DIRECTION_BINS_16,
            21 => &healpix::DIRECTION_BINS_21,
            32 => &healpix::DIRECTION_BINS_32,
            64 => &healpix::DIRECTION_BINS_64,
            84 => &healpix::DIRECTION_BINS_84,
            _ => unimplemented!(),
        };
        Self {
            directions: bins
                .iter()
                .map(|&[x, y, z]| Direction(MVec::new(x, y, z) * Dimensionless::dimensionless(1.0)))
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

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct DirectionsRng(StdRng);

fn get_rotation_matrix(axis: MVec, angle: f64) -> [[f64; 3]; 3] {
    let (x, y, z) = (axis.x, axis.y, axis.z);
    let cos = angle.cos();
    let sin = angle.sin();
    [
        [
            cos + x * x * (1.0 - cos),
            x * y * (1.0 - cos) - z * sin,
            x * z * (1.0 - cos) + y * sin,
        ],
        [
            y * x * (1.0 - cos) + z * sin,
            cos + y * y * (1.0 - cos),
            y * z * (1.0 - cos) - x * sin,
        ],
        [
            z * x * (1.0 - cos) - y * sin,
            z * y * (1.0 - cos) + x * sin,
            cos + z * z * (1.0 - cos),
        ],
    ]
}

fn get_random_rotation_matrix(rng: &mut StdRng) -> [[f64; 3]; 3] {
    let phi = rng.gen_range(0.0..(2.0 * PI));
    let rand: f64 = rng.gen_range(0.0..1.0);
    let theta = (2.0 * rand - 1.0).acos();
    let axis = MVec::new(
        phi.cos() * theta.sin(),
        phi.sin() * theta.sin(),
        theta.cos(),
    );
    let psi = rng.gen_range(0.0..(2.0 * PI));
    get_rotation_matrix(axis, psi)
}

fn multiply_by_matrix(vec: &mut MVec, matrix: &[[f64; 3]; 3]) {
    let (x, y, z) = (vec.x, vec.y, vec.z);
    vec.x = x * matrix[0][0] + y * matrix[0][1] + z * matrix[0][2];
    vec.y = x * matrix[1][0] + y * matrix[1][1] + z * matrix[1][2];
    vec.z = x * matrix[2][0] + y * matrix[2][1] + z * matrix[2][2];
}

// See nbubis' reply in https://math.stackexchange.com/questions/442418/random-generation-of-rotation-matrices
pub(super) fn rotate_directions_system(
    mut solver: NonSendMut<Option<Sweep<HydrogenOnly>>>,
    mut rng: ResMut<DirectionsRng>,
) {
    let solver = (*solver).as_mut().unwrap();
    let matrix = get_random_rotation_matrix(&mut rng);
    for dir in solver.directions.directions.iter_mut() {
        multiply_by_matrix(&mut dir.0 .0, &matrix)
    }
    panic!("Rotations currently mess up results in substepping runs.");
}

pub(super) fn init_directions_rng(sim: &mut Simulation) {
    const DIRECTIONS_RNG_SEED: u64 = 1337;
    sim.insert_resource(DirectionsRng(StdRng::seed_from_u64(DIRECTIONS_RNG_SEED)));
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::get_random_rotation_matrix;
    use super::multiply_by_matrix;
    use crate::test_utils::assert_float_is_close;
    use crate::units::MVec;
    use crate::voronoi::math::utils::determinant3x3;

    #[test]
    fn rotation_matrix_has_determinant_1() {
        let mut rng = StdRng::seed_from_u64(1337);
        for _ in 0..100 {
            let m = get_random_rotation_matrix(&mut rng);
            assert_float_is_close(determinant3x3(m), 1.0);
        }
    }

    // This is technically covered by the above test but oh well.
    #[test]
    fn rotation_matrix_preserves_vector_norm() {
        let mut rng = StdRng::seed_from_u64(1338);
        for _ in 0..100 {
            let m = get_random_rotation_matrix(&mut rng);
            let mut v = MVec::X;
            multiply_by_matrix(&mut v, &m);
            assert_float_is_close(v.length(), 1.0);
        }
    }
}
