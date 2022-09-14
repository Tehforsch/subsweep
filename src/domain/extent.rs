use mpi::traits::Equivalence;

use crate::units::Length;
use crate::units::VecLength;

#[derive(Default, Clone, Equivalence, PartialEq)]
pub struct Extent {
    pub min: VecLength,
    pub max: VecLength,
    pub center: VecLength,
}

impl Extent {
    pub fn new(min_x: Length, max_x: Length, min_y: Length, max_y: Length) -> Self {
        debug_assert!(min_x <= max_x);
        debug_assert!(min_y <= max_y);
        let min = VecLength::new(min_x, min_y);
        let max = VecLength::new(max_x, max_y);
        Self {
            min,
            max,
            center: (min + max) * 0.5,
        }
    }

    pub fn get_all_encompassing<'a>(extent: impl Iterator<Item = &'a Extent>) -> Option<Self> {
        Self::from_positions(
            extent.flat_map(|extent: &Extent| [&extent.min, &extent.max].into_iter()),
        )
    }

    /// Return an extent with slightly increased size
    /// but the same center
    pub fn pad(self) -> Self {
        let dist_to_min = self.min - self.center;
        let dist_to_max = self.max - self.center;
        const PADDING_FRACTION: f32 = 0.01;
        Self {
            min: self.center + dist_to_min * (1.0 + PADDING_FRACTION),
            max: self.center + dist_to_max * (1.0 + PADDING_FRACTION),
            center: self.center,
        }
    }

    pub fn center(&self) -> VecLength {
        VecLength::new(self.center.x(), self.center.y())
    }

    pub fn side_lengths(&self) -> VecLength {
        self.max - self.min
    }

    pub fn max_side_length(&self) -> Length {
        let side_length = self.side_lengths();
        side_length.x().max(&side_length.y())
    }

    pub fn from_positions<'a>(positions: impl Iterator<Item = &'a VecLength>) -> Option<Self> {
        let mut min_x = None;
        let mut max_x = None;
        let mut min_y = None;
        let mut max_y = None;
        let update_min = |x: &mut Option<Length>, y: Length| {
            if x.is_none() || y < x.unwrap() {
                *x = Some(y);
            }
        };
        let update_max = |x: &mut Option<Length>, y: Length| {
            if x.is_none() || y > x.unwrap() {
                *x = Some(y);
            }
        };
        for pos in positions {
            update_min(&mut min_x, pos.x());
            update_max(&mut max_x, pos.x());
            update_min(&mut min_y, pos.y());
            update_max(&mut max_y, pos.y());
        }
        Some(Self::new(min_x?, max_x?, min_y?, max_y?))
    }

    pub fn get_quadrant_index(&self, pos: &VecLength) -> usize {
        debug_assert!(self.contains(pos));
        match (pos.x() < self.center.x(), pos.y() < self.center.y()) {
            (true, true) => 0,
            (false, true) => 1,
            (false, false) => 2,
            (true, false) => 3,
        }
    }

    pub fn contains(&self, pos: &VecLength) -> bool {
        self.min.x() <= pos.x()
            && pos.x() <= self.max.x()
            && self.min.y() <= pos.y()
            && pos.y() <= self.max.y()
    }

    pub fn get_quadrants(&self) -> [Self; 4] {
        let lower_left = Self::new(self.min.x(), self.center.x(), self.min.y(), self.center.y());
        let lower_right = Self::new(self.center.x(), self.max.x(), self.min.y(), self.center.y());
        let upper_right = Self::new(self.center.x(), self.max.x(), self.center.y(), self.max.y());
        let upper_left = Self::new(self.min.x(), self.center.x(), self.center.y(), self.max.y());
        [lower_left, lower_right, upper_right, upper_left]
    }
}

impl std::fmt::Debug for Extent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Extent({:.3?} {:.3?})", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use crate::domain::Extent;
    use crate::units::Length;
    use crate::units::VecLength;

    fn assert_is_close(a: VecLength, b: Vec2) {
        const EPSILON: f32 = 1e-20;
        assert!((a - VecLength::meter(b.x, b.y)).length().unwrap_value() < EPSILON)
    }

    #[test]
    fn extent_quadrants() {
        let root_extent = Extent::new(
            Length::meter(-1.0),
            Length::meter(1.0),
            Length::meter(-2.0),
            Length::meter(2.0),
        );
        let quadrants = root_extent.get_quadrants();
        assert_is_close(quadrants[0].min, Vec2::new(-1.0, -2.0));
        assert_is_close(quadrants[0].max, Vec2::new(0.0, 0.0));

        assert_is_close(quadrants[1].min, Vec2::new(0.0, -2.0));
        assert_is_close(quadrants[1].max, Vec2::new(1.0, 0.0));

        assert_is_close(quadrants[2].min, Vec2::new(0.0, 0.0));
        assert_is_close(quadrants[2].max, Vec2::new(1.0, 2.0));

        assert_is_close(quadrants[3].min, Vec2::new(-1.0, 0.0));
        assert_is_close(quadrants[3].max, Vec2::new(0.0, 2.0));
    }

    #[test]
    fn extent_from_positions() {
        let positions = &[
            VecLength::meter(1.0, 0.0),
            VecLength::meter(-1.0, 0.0),
            VecLength::meter(0.0, -2.0),
            VecLength::meter(0.0, 2.0),
        ];
        let extent = Extent::from_positions(positions.iter()).unwrap();
        assert_is_close(extent.min, Vec2::new(-1.0, -2.0));
        assert_is_close(extent.max, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn extent_from_positions_is_none_with_zero_positions() {
        assert!(Extent::from_positions([].iter()).is_none());
    }

    #[test]
    fn extent_from_positions_is_none_with_particles_at_same_positions() {
        let positions = &[
            VecLength::meter(1.0, 0.0),
            VecLength::meter(1.0, 0.0),
            VecLength::meter(1.0, 0.0),
        ];
        assert!(Extent::from_positions(positions.iter()).is_none());
    }

    #[test]
    fn extent_from_particles_is_none_with_particles_in_a_line() {
        let positions = &[
            VecLength::meter(1.0, 0.0),
            VecLength::meter(2.0, 0.0),
            VecLength::meter(3.0, 0.0),
        ];
        assert!(Extent::from_positions(positions.iter()).is_none());
    }

    #[test]
    #[should_panic]
    fn invalid_extent() {
        Extent::new(
            Length::meter(1.0),
            Length::meter(-1.0),
            Length::meter(0.0),
            Length::meter(1.0),
        );
    }
}
