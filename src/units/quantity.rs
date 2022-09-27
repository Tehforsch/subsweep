use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use glam::DVec2;

use super::dimension::Dimension;
use super::UNIT_NAMES;
use crate::units::NONE;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
pub struct Quantity<S: 'static, const D: Dimension>(pub(super) S);

impl<S> Quantity<S, { NONE }> {
    /// Get the value of a dimensionless quantity
    pub fn value(&self) -> &S {
        &self.0
    }
}

impl<S> std::ops::Deref for Quantity<S, NONE> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> std::ops::DerefMut for Quantity<S, NONE> {
    fn deref_mut(&mut self) -> &mut S {
        &mut self.0
    }
}

impl<S, const D: Dimension> Quantity<S, D>
where
    S: Clone,
{
    /// Unwrap the value of a quantity, regardless of whether
    /// it is dimensionless or not. Use this carefully, since the
    /// result depends on the underlying base units
    pub fn unwrap_value(self) -> S {
        self.0
    }

    pub fn in_units(self, other: Quantity<f64, D>) -> S
    where
        S: Div<f64, Output = S>,
        Quantity<S, { D.dimension_div(D) }>:,
    {
        (self / other).unwrap_value()
    }
}

impl<const D: Dimension> Quantity<f64, D> {
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl<S, const D: Dimension> Add for Quantity<S, D>
where
    S: Add<Output = S>,
{
    type Output = Quantity<S, D>;

    fn add(self, rhs: Self) -> Self::Output {
        Quantity::<S, D>(self.0 + rhs.0)
    }
}

impl<S, const D: Dimension> Neg for Quantity<S, D>
where
    S: Neg<Output = S>,
{
    type Output = Quantity<S, D>;

    fn neg(self) -> Self::Output {
        Quantity::<S, D>(-self.0)
    }
}

impl<S, const D: Dimension> AddAssign for Quantity<S, D>
where
    S: AddAssign<S>,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<S, const D: Dimension> Sub for Quantity<S, D>
where
    S: Sub<Output = S>,
{
    type Output = Quantity<S, D>;

    fn sub(self, rhs: Self) -> Self::Output {
        Quantity::<S, D>(self.0 - rhs.0)
    }
}

impl<S, const D: Dimension> SubAssign for Quantity<S, D>
where
    S: SubAssign<S>,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<S, const D: Dimension> Mul<f64> for Quantity<S, D>
where
    S: Mul<f64, Output = S>,
{
    type Output = Quantity<S, D>;

    fn mul(self, rhs: f64) -> Self::Output {
        Quantity(self.0 * rhs)
    }
}

impl<S, const D: Dimension> Mul<Quantity<S, D>> for f64
where
    f64: Mul<S, Output = S>,
{
    type Output = Quantity<S, D>;

    fn mul(self, rhs: Quantity<S, D>) -> Self::Output {
        Quantity(self * rhs.0)
    }
}

impl<S, const D: Dimension> Div<f64> for Quantity<S, D>
where
    S: Div<f64, Output = S>,
{
    type Output = Quantity<S, D>;

    fn div(self, rhs: f64) -> Self::Output {
        Quantity(self.0 / rhs)
    }
}

impl<S, const D: Dimension> Div<Quantity<S, D>> for f64
where
    Quantity<S, { D.dimension_inv() }>:,
    f64: Div<S, Output = S>,
{
    type Output = Quantity<S, { D.dimension_inv() }>;

    fn div(self, rhs: Quantity<S, D>) -> Self::Output {
        Quantity(self / rhs.0)
    }
}

impl<SL, SR, const DL: Dimension, const DR: Dimension> Mul<Quantity<SR, DR>> for Quantity<SL, DL>
where
    Quantity<SL, { DL.dimension_mul(DR) }>:,
    SL: Mul<SR, Output = SL>,
{
    type Output = Quantity<SL, { DL.dimension_mul(DR) }>;

    fn mul(self, rhs: Quantity<SR, DR>) -> Self::Output {
        Quantity(self.0 * rhs.0)
    }
}

impl<SL, SR, const DL: Dimension, const DR: Dimension> Div<Quantity<SR, DR>> for Quantity<SL, DL>
where
    Quantity<SL, { DL.dimension_div(DR) }>:,
    SL: Div<SR, Output = SL>,
{
    type Output = Quantity<SL, { DL.dimension_div(DR) }>;

    fn div(self, rhs: Quantity<SR, DR>) -> Self::Output {
        Quantity(self.0 / rhs.0)
    }
}

impl<S, const D: Dimension> Quantity<S, D>
where
    Quantity<S, { D.dimension_div(D) }>:,
    S: Div<S, Output = S> + Copy,
{
    pub fn to_value(&self, unit: impl Fn(f64) -> Quantity<S, D>) -> S {
        (*self / unit(1.0)).unwrap_value()
    }
}

impl<S> std::fmt::Display for Quantity<S, { NONE }>
where
    S: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl<const D: Dimension> std::fmt::Debug for Quantity<f64, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit_name = UNIT_NAMES
            .iter()
            .filter(|(d, _, _)| d == &D)
            .filter(|(_, _, val)| *val == 1.0)
            .map(|(_, name, _)| name)
            .next()
            .unwrap_or(&"unknown unit");
        self.0.fmt(f).and_then(|_| write!(f, " {}", unit_name))
    }
}

impl<const D: Dimension> std::fmt::Debug for Quantity<DVec2, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit_name = UNIT_NAMES
            .iter()
            .filter(|(d, _, _)| d == &D)
            .filter(|(_, _, val)| *val == 1.0)
            .map(|(_, name, _)| name)
            .next()
            .unwrap_or(&"unknown unit");
        write!(f, "[")?;
        self.0.x.fmt(f)?;
        write!(f, " ")?;
        self.0.y.fmt(f)?;
        write!(f, "] {}", unit_name)
    }
}
