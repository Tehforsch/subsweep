use crate::units::Length;
use crate::units::VecLength;

#[derive(Clone, Debug)]
pub struct Extents {
    pub x_min: Length,
    pub x_max: Length,
    pub y_min: Length,
    pub y_max: Length,
    pub x_center: Length,
    pub y_center: Length,
}

impl Extents {
    pub fn new(x_min: Length, x_max: Length, y_min: Length, y_max: Length) -> Self {
        debug_assert!(x_min < x_max);
        debug_assert!(y_min < y_max);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
            x_center: (x_min + x_max) * 0.5,
            y_center: (y_min + y_max) * 0.5,
        }
    }

    pub fn center(&self) -> VecLength {
        VecLength::new(self.x_center, self.y_center)
    }

    pub fn max_side_length(&self) -> Length {
        (self.x_max - self.x_min).max(&(self.y_max - self.y_min))
    }

    pub fn from_positions<'a>(positions: impl Iterator<Item = &'a VecLength>) -> Option<Self> {
        let mut x_min = None;
        let mut x_max = None;
        let mut y_min = None;
        let mut y_max = None;
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
            update_min(&mut x_min, pos.x());
            update_max(&mut x_max, pos.x());
            update_min(&mut y_min, pos.y());
            update_max(&mut y_max, pos.y());
        }
        if x_min == x_max || y_min == y_max {
            None
        } else {
            Some(Self::new(x_min?, x_max?, y_min?, y_max?))
        }
    }

    pub fn get_quadrant_index(&self, pos: &VecLength) -> usize {
        debug_assert!(self.contains(pos));
        match (pos.x() < self.x_center, pos.y() < self.y_center) {
            (true, true) => 0,
            (false, true) => 1,
            (false, false) => 2,
            (true, false) => 3,
        }
    }

    pub fn contains(&self, pos: &VecLength) -> bool {
        self.x_min <= pos.x()
            && pos.x() <= self.x_max
            && self.y_min <= pos.y()
            && pos.y() <= self.y_max
    }

    pub fn get_quadrants(&self) -> [Self; 4] {
        let lower_left = Self::new(self.x_min, self.x_center, self.y_min, self.y_center);
        let lower_right = Self::new(self.x_center, self.x_max, self.y_min, self.y_center);
        let upper_right = Self::new(self.x_center, self.x_max, self.y_center, self.y_max);
        let upper_left = Self::new(self.x_min, self.x_center, self.y_center, self.y_max);
        [lower_left, lower_right, upper_right, upper_left]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Extents;
    use crate::units::Length;
    use crate::units::VecLength;

    fn assert_is_close(x: Length, y: f32) {
        const EPSILON: f32 = 1e-20;
        assert!((x - Length::meter(y)).unwrap_value().abs() < EPSILON)
    }

    #[test]
    fn extent_quadrants() {
        let root_extents = Extents::new(
            Length::meter(-1.0),
            Length::meter(1.0),
            Length::meter(-2.0),
            Length::meter(2.0),
        );
        let quadrants = root_extents.get_quadrants();
        assert_is_close(quadrants[0].x_min, -1.0);
        assert_is_close(quadrants[0].x_max, 0.0);
        assert_is_close(quadrants[0].y_min, -2.0);
        assert_is_close(quadrants[0].y_max, 0.0);

        assert_is_close(quadrants[1].x_min, 0.0);
        assert_is_close(quadrants[1].x_max, 1.0);
        assert_is_close(quadrants[1].y_min, -2.0);
        assert_is_close(quadrants[1].y_max, 0.0);

        assert_is_close(quadrants[2].x_min, 0.0);
        assert_is_close(quadrants[2].x_max, 1.0);
        assert_is_close(quadrants[2].y_min, 0.0);
        assert_is_close(quadrants[2].y_max, 2.0);

        assert_is_close(quadrants[3].x_min, -1.0);
        assert_is_close(quadrants[3].x_max, 0.0);
        assert_is_close(quadrants[3].y_min, 0.0);
        assert_is_close(quadrants[3].y_max, 2.0);
    }

    #[test]
    fn extent_from_positions() {
        let positions = &[
            VecLength::meter(1.0, 0.0),
            VecLength::meter(-1.0, 0.0),
            VecLength::meter(0.0, -2.0),
            VecLength::meter(0.0, 2.0),
        ];
        let extents = Extents::from_positions(positions.iter()).unwrap();
        assert_is_close(extents.x_min, -1.0);
        assert_is_close(extents.x_max, 1.0);
        assert_is_close(extents.y_min, -2.0);
        assert_is_close(extents.y_max, 2.0);
    }

    #[test]
    fn extent_from_positions_is_none_with_zero_positions() {
        assert!(Extents::from_positions([].iter()).is_none());
    }

    #[test]
    fn extent_from_positions_is_none_with_particles_at_same_positions() {
        let positions = &[
            VecLength::meter(1.0, 0.0),
            VecLength::meter(1.0, 0.0),
            VecLength::meter(1.0, 0.0),
        ];
        assert!(Extents::from_positions(positions.iter()).is_none());
    }

    #[test]
    fn extent_from_particles_is_none_with_particles_in_a_line() {
        let positions = &[
            VecLength::meter(1.0, 0.0),
            VecLength::meter(2.0, 0.0),
            VecLength::meter(3.0, 0.0),
        ];
        assert!(Extents::from_positions(positions.iter()).is_none());
    }

    #[test]
    #[should_panic]
    fn invalid_extents() {
        Extents::new(
            Length::meter(1.0),
            Length::meter(-1.0),
            Length::meter(0.0),
            Length::meter(1.0),
        );
    }
}
