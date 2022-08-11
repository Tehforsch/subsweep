#[derive(PartialEq, Eq, Debug)]
pub struct Dimension {
    pub length: usize,
    pub time: usize,
    pub mass: usize,
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
}
