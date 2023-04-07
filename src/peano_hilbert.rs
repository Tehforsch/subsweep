use glam::DVec2;
use mpi::traits::Equivalence;

// These values are for 3d, but I'll use them for 2D as well, since it
// doesn't really matter there anyways
pub const NUM_BITS_PER_DIMENSION: u32 = 21;
const NUM_SUBDIVISIONS: u64 = 2u64.pow(NUM_BITS_PER_DIMENSION);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Equivalence)]
pub struct PeanoHilbertKey(pub u64);

fn get_integer_position(pos: DVec2) -> (u64, u64) {
    (
        (pos.x * NUM_SUBDIVISIONS as f64) as u64,
        (pos.y * NUM_SUBDIVISIONS as f64) as u64,
    )
}

impl PeanoHilbertKey {
    pub fn from_point_and_min_max_2d(pos: DVec2, min: DVec2, max: DVec2) -> Self {
        let min_padded = min - (max - min) * 0.001;
        let max_padded = max + (max - min) * 0.001;
        Self::new((pos - min_padded) / (max_padded - min_padded))
    }

    fn new(pos: DVec2) -> Self {
        let integer_pos = get_integer_position(pos);
        Self::from_integer_pos(integer_pos)
    }

    // Source: https://en.wikipedia.org/wiki/Hilbert_curve
    fn from_integer_pos((mut x, mut y): (u64, u64)) -> Self {
        let mut s = NUM_SUBDIVISIONS / 2;
        let mut d = 0;
        while s > 0 {
            let rx = ((x & s) > 0) as u64;
            let ry = ((y & s) > 0) as u64;
            d += s * s * ((3 * rx) ^ ry);
            Self::rot(NUM_SUBDIVISIONS, &mut x, &mut y, rx, ry);
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

#[cfg(test)]
mod tests {
    use super::PeanoHilbertKey;
    use super::NUM_SUBDIVISIONS;

    impl PeanoHilbertKey {
        fn to_integer_pos(&self) -> (u64, u64) {
            let mut t = self.0;
            let mut x = 0;
            let mut y = 0;
            let mut s = 1;
            while s < NUM_SUBDIVISIONS {
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
                let d = PeanoHilbertKey::from_integer_pos((x, y));
                assert_eq!(d.to_integer_pos(), (x, y));
            }
        }
    }
}
