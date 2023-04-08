use glam::DVec2;
use glam::DVec3;
use mpi::datatype::UserDatatype;
use mpi::traits::Equivalence;
use mpi::Address;

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
    pub fn from_point_and_min_max(pos: DVec2, min: DVec2, max: DVec2) -> Self {
        let min_padded = min - (max - min) * 0.001;
        let max_padded = max + (max - min) * 0.001;
        Self::from_scaled_vec((pos - min_padded) / (max_padded - min_padded))
    }

    fn from_scaled_vec(pos: DVec2) -> Self {
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
    pub fn from_point_and_min_max(pos: DVec3, min: DVec3, max: DVec3) -> Self {
        let min_padded = min - (max - min) * 0.001;
        let max_padded = max + (max - min) * 0.001;
        Self::from_scaled_vec((pos - min_padded) / (max_padded - min_padded))
    }

    fn from_scaled_vec(pos: DVec3) -> Self {
        let integer_pos = get_integer_position_3d(pos);
        Self::from_integer_pos(integer_pos)
    }

    fn from_integer_pos((x, y, z): (u128, u128, u128)) -> Self {
        let mut rotation: usize = 0;
        let mut key: u128 = 0;
        let mut mask = 1 << (NUM_BITS_PER_DIMENSION_3D - 1);
        while mask > 0 {
            let xmask = if x & mask != 0 { 4 } else { 0 };
            let ymask = if y & mask != 0 { 2 } else { 0 };
            let zmask = if z & mask != 0 { 1 } else { 0 };
            let pix = xmask | ymask | zmask;

            key <<= 3;
            key |= SUBPIX_TABLE[rotation][pix];
            rotation = ROTATION_TABLE[rotation][pix];

            mask >>= 1
        }
        return PeanoKey3d(key);
    }
}

// Source: Arepo code / Martin Reinecke
const ROTATION_TABLE: [[usize; 8]; 48] = [
    [36, 28, 25, 27, 10, 10, 25, 27],
    [29, 11, 24, 24, 37, 11, 26, 26],
    [8, 8, 25, 27, 30, 38, 25, 27],
    [9, 39, 24, 24, 9, 31, 26, 26],
    [40, 24, 44, 32, 40, 6, 44, 6],
    [25, 7, 33, 7, 41, 41, 45, 45],
    [4, 42, 4, 46, 26, 42, 34, 46],
    [43, 43, 47, 47, 5, 27, 5, 35],
    [33, 35, 36, 28, 33, 35, 2, 2],
    [32, 32, 29, 3, 34, 34, 37, 3],
    [33, 35, 0, 0, 33, 35, 30, 38],
    [32, 32, 1, 39, 34, 34, 1, 31],
    [24, 42, 32, 46, 14, 42, 14, 46],
    [43, 43, 47, 47, 25, 15, 33, 15],
    [40, 12, 44, 12, 40, 26, 44, 34],
    [13, 27, 13, 35, 41, 41, 45, 45],
    [28, 41, 28, 22, 38, 43, 38, 22],
    [42, 40, 23, 23, 29, 39, 29, 39],
    [41, 36, 20, 36, 43, 30, 20, 30],
    [37, 31, 37, 31, 42, 40, 21, 21],
    [28, 18, 28, 45, 38, 18, 38, 47],
    [19, 19, 46, 44, 29, 39, 29, 39],
    [16, 36, 45, 36, 16, 30, 47, 30],
    [37, 31, 37, 31, 17, 17, 46, 44],
    [12, 4, 1, 3, 34, 34, 1, 3],
    [5, 35, 0, 0, 13, 35, 2, 2],
    [32, 32, 1, 3, 6, 14, 1, 3],
    [33, 15, 0, 0, 33, 7, 2, 2],
    [16, 0, 20, 8, 16, 30, 20, 30],
    [1, 31, 9, 31, 17, 17, 21, 21],
    [28, 18, 28, 22, 2, 18, 10, 22],
    [19, 19, 23, 23, 29, 3, 29, 11],
    [9, 11, 12, 4, 9, 11, 26, 26],
    [8, 8, 5, 27, 10, 10, 13, 27],
    [9, 11, 24, 24, 9, 11, 6, 14],
    [8, 8, 25, 15, 10, 10, 25, 7],
    [0, 18, 8, 22, 38, 18, 38, 22],
    [19, 19, 23, 23, 1, 39, 9, 39],
    [16, 36, 20, 36, 16, 2, 20, 10],
    [37, 3, 37, 11, 17, 17, 21, 21],
    [4, 17, 4, 46, 14, 19, 14, 46],
    [18, 16, 47, 47, 5, 15, 5, 15],
    [17, 12, 44, 12, 19, 6, 44, 6],
    [13, 7, 13, 7, 18, 16, 45, 45],
    [4, 42, 4, 21, 14, 42, 14, 23],
    [43, 43, 22, 20, 5, 15, 5, 15],
    [40, 12, 21, 12, 40, 6, 23, 6],
    [13, 7, 13, 7, 41, 41, 22, 20],
];

const SUBPIX_TABLE: [[u128; 8]; 48] = [
    [0, 7, 1, 6, 3, 4, 2, 5],
    [7, 4, 6, 5, 0, 3, 1, 2],
    [4, 3, 5, 2, 7, 0, 6, 1],
    [3, 0, 2, 1, 4, 7, 5, 6],
    [1, 0, 6, 7, 2, 3, 5, 4],
    [0, 3, 7, 4, 1, 2, 6, 5],
    [3, 2, 4, 5, 0, 1, 7, 6],
    [2, 1, 5, 6, 3, 0, 4, 7],
    [6, 1, 7, 0, 5, 2, 4, 3],
    [1, 2, 0, 3, 6, 5, 7, 4],
    [2, 5, 3, 4, 1, 6, 0, 7],
    [5, 6, 4, 7, 2, 1, 3, 0],
    [7, 6, 0, 1, 4, 5, 3, 2],
    [6, 5, 1, 2, 7, 4, 0, 3],
    [5, 4, 2, 3, 6, 7, 1, 0],
    [4, 7, 3, 0, 5, 6, 2, 1],
    [6, 7, 5, 4, 1, 0, 2, 3],
    [7, 0, 4, 3, 6, 1, 5, 2],
    [0, 1, 3, 2, 7, 6, 4, 5],
    [1, 6, 2, 5, 0, 7, 3, 4],
    [2, 3, 1, 0, 5, 4, 6, 7],
    [3, 4, 0, 7, 2, 5, 1, 6],
    [4, 5, 7, 6, 3, 2, 0, 1],
    [5, 2, 6, 1, 4, 3, 7, 0],
    [7, 0, 6, 1, 4, 3, 5, 2],
    [0, 3, 1, 2, 7, 4, 6, 5],
    [3, 4, 2, 5, 0, 7, 1, 6],
    [4, 7, 5, 6, 3, 0, 2, 1],
    [6, 7, 1, 0, 5, 4, 2, 3],
    [7, 4, 0, 3, 6, 5, 1, 2],
    [4, 5, 3, 2, 7, 6, 0, 1],
    [5, 6, 2, 1, 4, 7, 3, 0],
    [1, 6, 0, 7, 2, 5, 3, 4],
    [6, 5, 7, 4, 1, 2, 0, 3],
    [5, 2, 4, 3, 6, 1, 7, 0],
    [2, 1, 3, 0, 5, 6, 4, 7],
    [0, 1, 7, 6, 3, 2, 4, 5],
    [1, 2, 6, 5, 0, 3, 7, 4],
    [2, 3, 5, 4, 1, 0, 6, 7],
    [3, 0, 4, 7, 2, 1, 5, 6],
    [1, 0, 2, 3, 6, 7, 5, 4],
    [0, 7, 3, 4, 1, 6, 2, 5],
    [7, 6, 4, 5, 0, 1, 3, 2],
    [6, 1, 5, 2, 7, 0, 4, 3],
    [5, 4, 6, 7, 2, 3, 1, 0],
    [4, 3, 7, 0, 5, 2, 6, 1],
    [3, 2, 0, 1, 4, 5, 7, 6],
    [2, 5, 1, 6, 3, 4, 0, 7],
];

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
