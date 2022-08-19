#[derive(PartialEq, Eq, Debug, Clone)]
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
}
