use glam::DVec2;
use glam::DVec3;
use mpi::datatype::UserDatatype;
use mpi::traits::Equivalence;
use mpi::Address;

use crate::domain::Extent;
use crate::units::VecLength;

pub const NUM_BITS_PER_DIMENSION_2D: u32 = 64 / 2;
const NUM_SUBDIVISIONS_2D: u64 = 2u64.pow(NUM_BITS_PER_DIMENSION_2D);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Equivalence)]
pub struct PeanoKey2d(pub u64);

pub const NUM_BITS_PER_DIMENSION_3D: u32 = 128 / 3;
const NUM_SUBDIVISIONS_3D: u64 = 2u64.pow(NUM_BITS_PER_DIMENSION_3D);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeanoKey3d(pub u128);

// Yikes. Don't know how to natively send u128 via MPI.
unsafe impl Equivalence for PeanoKey3d {
    type Out = UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        UserDatatype::structured(
            &[1],
            &[0 as Address],
            &[UserDatatype::contiguous(2, &u64::equivalent_datatype())],
        )
    }
}

fn get_integer_position_2d(pos: DVec2) -> (u64, u64) {
    (
        (pos.x * NUM_SUBDIVISIONS_2D as f64) as u64,
        (pos.y * NUM_SUBDIVISIONS_2D as f64) as u64,
    )
}

fn get_integer_position_3d(pos: DVec3) -> (u128, u128, u128) {
    (
        (pos.x * NUM_SUBDIVISIONS_3D as f64) as u128,
        (pos.y * NUM_SUBDIVISIONS_3D as f64) as u128,
        (pos.z * NUM_SUBDIVISIONS_3D as f64) as u128,
    )
}

impl PeanoKey2d {
    pub fn from_point_and_min_max_2d(pos: DVec2, min: DVec2, max: DVec2) -> Self {
        let min_padded = min - (max - min) * 0.001;
        let max_padded = max + (max - min) * 0.001;
        Self::new((pos - min_padded) / (max_padded - min_padded))
    }

    fn new(pos: DVec2) -> Self {
        let integer_pos = get_integer_position_2d(pos);
        Self::from_integer_pos(integer_pos)
    }

    // Source: https://en.wikipedia.org/wiki/Hilbert_curve
    fn from_integer_pos((mut x, mut y): (u64, u64)) -> Self {
        let mut s = NUM_SUBDIVISIONS_2D / 2;
        let mut d = 0;
        while s > 0 {
            let rx = ((x & s) > 0) as u64;
            let ry = ((y & s) > 0) as u64;
            d += s * s * ((3 * rx) ^ ry);
            Self::rot(NUM_SUBDIVISIONS_2D, &mut x, &mut y, rx, ry);
            s /= 2;
        }
        Self(d)
    }

    fn rot(n: u64, x: &mut u64, y: &mut u64, rx: u64, ry: u64) {
        if ry == 0 {
            if rx == 1 {
                *x = (n - 1) - *x;
                *y = (n - 1) - *y;
            }
            std::mem::swap(x, y);
        }
    }
}

impl PeanoKey3d {
    pub fn from_point_and_extent(pos: VecLength, extent: &Extent) -> Self {
        let min_padded = extent.min - (extent.max - extent.min) * 0.001;
        let max_padded = extent.max + (extent.max - extent.min) * 0.001;
        let vec3d =
            (pos - min_padded).value_unchecked() / (max_padded - min_padded).value_unchecked();
        Self::new(vec3d)
    }

    fn new(pos: DVec3) -> Self {
        let integer_pos = get_integer_position_3d(pos);
        Self::from_integer_pos(integer_pos)
    }

    fn from_integer_pos((_x, _y, _z): (u128, u128, u128)) -> Self {
        todo!()
        // let mut s = NUM_SUBDIVISIONS_2D / 2;
        // let mut d = 0;
        // while s > 0 {
        //     let rx = ((x & s) > 0) as u128;
        //     let ry = ((y & s) > 0) as u128;
        //     d += s * s * ((3 * rx) ^ ry);
        //     Self::rot(NUM_SUBDIVISIONS_2D, &mut x, &mut y, rx, ry);
        //     s /= 2;
        // }
        // Self(d)
    }

    // fn rot(n: u64, x: &mut u64, y: &mut u64, rx: u64, ry: u64) {
    //     if ry == 0 {
    //         if rx == 1 {
    //             *x = (n - 1) - *x;
    //             *y = (n - 1) - *y;
    //         }
    //         std::mem::swap(x, y);
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::PeanoKey2d;
    use super::NUM_SUBDIVISIONS_2D;

    impl PeanoKey2d {
        fn to_integer_pos(&self) -> (u64, u64) {
            let mut t = self.0;
            let mut x = 0;
            let mut y = 0;
            let mut s = 1;
            while s < NUM_SUBDIVISIONS_2D {
                let rx = 1 & (t / 2);
                let ry = 1 & (t ^ rx);
                Self::rot(s, &mut x, &mut y, rx, ry);
                x += s * rx;
                y += s * ry;
                t /= 4;
                s *= 2;
            }
            (x, y)
        }
    }
    #[test]
    fn peano_hilbert_map_is_isomorphic() {
        for x in 0..30 {
            for y in 0..30 {
                let d = PeanoKey2d::from_integer_pos((x, y));
                assert_eq!(d.to_integer_pos(), (x, y));
            }
        }
    }
}
