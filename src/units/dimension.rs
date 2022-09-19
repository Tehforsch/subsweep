const MASS_TO_SI: f64 = 1.0;
const LENGTH_TO_SI: f64 = 1.0;
const TIME_TO_SI: f64 = 1.0;

#[derive(PartialEq, Eq, Debug, Clone)]
#[non_exhaustive] // Prevents clippy warnings due to ..NONE update in quantities macro
pub struct Dimension {
    pub length: i32,
    pub time: i32,
    pub mass: i32,
}

impl Dimension {
    pub const fn dimension_mul(self, rhs: Self) -> Self {
        Self {
            length: self.length + rhs.length,
            mass: self.mass + rhs.mass,
            time: self.time + rhs.time,
        }
    }

    pub const fn dimension_div(self, rhs: Self) -> Self {
        Self {
            length: self.length - rhs.length,
            mass: self.mass - rhs.mass,
            time: self.time - rhs.time,
        }
    }

    pub const fn dimension_powi(self, rhs: i32) -> Self {
        Self {
            length: self.length * rhs,
            mass: self.mass * rhs,
            time: self.time * rhs,
        }
    }

    /// Get the base conversion factor of this dimension
    /// into SI units. As of now, this is always 1.0
    /// but will change if the base units are changed
    /// from SI to anything else
    pub fn base_conversion_factor(self) -> f64 {
        (LENGTH_TO_SI).powi(self.length)
            * (TIME_TO_SI).powi(self.time)
            * (MASS_TO_SI).powi(self.mass)
    }
}
