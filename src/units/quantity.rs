use super::dimension::Dimension;
use crate::units::NONE;
use std::ops::{Add, Div, Mul};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity<const U: Dimension>(pub(super) f64);

impl<const U: Dimension> Quantity<U> {
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}

impl Quantity<{ NONE }> {
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl<const U: Dimension> Add for Quantity<U> {
    type Output = Quantity<U>;

    fn add(self, rhs: Self) -> Self::Output {
        Quantity::<U>(self.0 + rhs.0)
    }
}

impl<const U: Dimension> Mul<f64> for Quantity<U> {
    type Output = Quantity<U>;

    fn mul(self, rhs: f64) -> Self::Output {
        Quantity(self.0 * rhs)
    }
}

impl<const U: Dimension> Mul<Quantity<U>> for f64 {
    type Output = Quantity<U>;

    fn mul(self, rhs: Quantity<U>) -> Self::Output {
        Quantity(self * rhs.0)
    }
}

impl<const UL: Dimension, const UR: Dimension> Mul<Quantity<UR>> for Quantity<UL>
where
    Quantity<{ UL.dimension_mul(UR) }>:,
{
    type Output = Quantity<{ UL.dimension_mul(UR) }>;

    fn mul(self, rhs: Quantity<UR>) -> Self::Output {
        Quantity(self.0 * rhs.0)
    }
}

impl<const UL: Dimension, const UR: Dimension> Div<Quantity<UR>> for Quantity<UL>
where
    Quantity<{ UL.dimension_div(UR) }>:,
{
    type Output = Quantity<{ UL.dimension_div(UR) }>;

    fn div(self, rhs: Quantity<UR>) -> Self::Output {
        Quantity(self.0 / rhs.0)
    }
}
