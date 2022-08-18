use crate::position::Position;
use crate::units::f32::Length;

#[derive(Clone, Debug)]
pub struct Extents {
    pub x_min: Length,
    pub x_max: Length,
    pub y_min: Length,
    pub y_max: Length,
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
        }
    }

    pub fn contains(&self, pos: &Position) -> bool {
        self.x_min <= pos.0.x()
            && pos.0.x() < self.x_max
            && self.y_min <= pos.0.y()
            && pos.0.y() < self.y_max
    }

    pub fn get_quadrants(&self) -> [Self; 4] {
        let center_x = (self.x_min + self.x_max) * 0.5;
        let center_y = (self.y_min + self.y_max) * 0.5;
        let lower_left = Extents {
            x_min: self.x_min,
            x_max: center_x,
            y_min: self.y_min,
            y_max: center_y,
        };
        let lower_right = Extents {
            x_min: center_x,
            x_max: self.x_max,
            y_min: self.y_min,
            y_max: center_y,
        };
        let upper_right = Extents {
            x_min: center_x,
            x_max: self.x_max,
            y_min: center_y,
            y_max: self.y_max,
        };
        let upper_left = Extents {
            x_min: self.x_min,
            x_max: center_x,
            y_min: center_y,
            y_max: self.y_max,
        };
        [lower_left, lower_right, upper_right, upper_left]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Extents;
    use crate::units::f32::meter;
    use crate::units::f32::Length;

    fn assert_is_close(x: Length, y: f32) {
        const EPSILON: f32 = 1e-20;
        assert!((x - meter(y)).unwrap_value().abs() < EPSILON)
    }

    #[test]
    fn extent_quadrants() {
        let root_extents = Extents::new(meter(-1.0), meter(1.0), meter(-2.0), meter(2.0));
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
}
