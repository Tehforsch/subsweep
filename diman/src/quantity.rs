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
use crate::dimension::NONE;

macro_rules! quantity {
    ($quantity: ident, $dimension: ident, $dimensionless_const: ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
        pub struct $quantity<S: 'static, const D: $dimension>(pub(crate) S);

        impl<S> $quantity<S, { $dimensionless_const }> {
            /// Get the value of a dimensionless quantity
            pub fn value(&self) -> &S {
                &self.0
            }
        }

        impl<S> std::ops::Deref for $quantity<S, $dimensionless_const> {
            type Target = S;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<S> std::ops::DerefMut for $quantity<S, $dimensionless_const> {
            fn deref_mut(&mut self) -> &mut S {
                &mut self.0
            }
        }

        impl<S, const D: $dimension> $quantity<S, D>
        where
            S: Clone,
        {
            /// Unwrap the value of a quantity, regardless of whether
            /// it is dimensionless or not. Use this carefully, since the
            /// result depends on the underlying base units
            pub fn unwrap_value(self) -> S {
                self.0
            }

            /// Create a new quantity for the dimension with a given value.
            /// Use carefully, since the constructed quantity depends on the
            /// used base units.
            pub const fn new_unchecked(s: S) -> Self {
                Self(s)
            }

            pub fn in_units(self, other: $quantity<f64, D>) -> S
            where
                S: Div<f64, Output = S>,
                $quantity<S, { D.dimension_div(D) }>:,
            {
                (self / other).unwrap_value()
            }
        }

        impl<S, const D: $dimension> Add for $quantity<S, D>
        where
            S: Add<Output = S>,
        {
            type Output = $quantity<S, D>;

            fn add(self, rhs: Self) -> Self::Output {
                $quantity::<S, D>(self.0 + rhs.0)
            }
        }

        impl<S, const D: $dimension> AddAssign for $quantity<S, D>
        where
            S: AddAssign<S>,
        {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<S, const D: $dimension> Sub for $quantity<S, D>
        where
            S: Sub<Output = S>,
        {
            type Output = $quantity<S, D>;

            fn sub(self, rhs: Self) -> Self::Output {
                $quantity::<S, D>(self.0 - rhs.0)
            }
        }

        impl<S, const D: $dimension> SubAssign for $quantity<S, D>
        where
            S: SubAssign<S>,
        {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<S, const D: $dimension> Neg for $quantity<S, D>
        where
            S: Neg<Output = S>,
        {
            type Output = $quantity<S, D>;

            fn neg(self) -> Self::Output {
                $quantity::<S, D>(-self.0)
            }
        }
        impl<S, const D: $dimension> Mul<f64> for $quantity<S, D>
        where
            S: Mul<f64, Output = S>,
        {
            type Output = $quantity<S, D>;

            fn mul(self, rhs: f64) -> Self::Output {
                $quantity(self.0 * rhs)
            }
        }

        impl<S, const D: $dimension> Mul<$quantity<S, D>> for f64
        where
            f64: Mul<S, Output = S>,
        {
            type Output = $quantity<S, D>;

            fn mul(self, rhs: $quantity<S, D>) -> Self::Output {
                $quantity(self * rhs.0)
            }
        }

        impl<S, const D: $dimension> Div<f64> for $quantity<S, D>
        where
            S: Div<f64, Output = S>,
        {
            type Output = $quantity<S, D>;

            fn div(self, rhs: f64) -> Self::Output {
                $quantity(self.0 / rhs)
            }
        }

        impl<S, const D: $dimension> Div<$quantity<S, D>> for f64
        where
            $quantity<S, { D.dimension_inv() }>:,
            f64: Div<S, Output = S>,
        {
            type Output = $quantity<S, { D.dimension_inv() }>;

            fn div(self, rhs: $quantity<S, D>) -> Self::Output {
                $quantity(self / rhs.0)
            }
        }

        impl<SL, SR, const DL: $dimension, const DR: $dimension> Mul<$quantity<SR, DR>>
            for $quantity<SL, DL>
        where
            $quantity<SL, { DL.dimension_mul(DR) }>:,
            SL: Mul<SR, Output = SL>,
        {
            type Output = $quantity<SL, { DL.dimension_mul(DR) }>;

            fn mul(self, rhs: $quantity<SR, DR>) -> Self::Output {
                $quantity(self.0 * rhs.0)
            }
        }

        impl<SL, SR, const DL: $dimension, const DR: $dimension> Div<$quantity<SR, DR>>
            for $quantity<SL, DL>
        where
            $quantity<SL, { DL.dimension_div(DR) }>:,
            SL: Div<SR, Output = SL>,
        {
            type Output = $quantity<SL, { DL.dimension_div(DR) }>;

            fn div(self, rhs: $quantity<SR, DR>) -> Self::Output {
                $quantity(self.0 / rhs.0)
            }
        }
        impl<S> std::fmt::Display for $quantity<S, { $dimensionless_const }>
        where
            S: std::fmt::Display,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.value())
            }
        }

        impl<const D: $dimension> std::fmt::Debug for $quantity<f64, D> {
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

        impl<const D: $dimension> std::fmt::Debug for $quantity<DVec2, D> {
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
    };
}

quantity!(Quantity, Dimension, NONE);

impl<const D: Dimension> Quantity<f64, D> {
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}
