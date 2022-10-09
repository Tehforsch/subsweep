#[macro_export]
macro_rules! impl_float_methods {
    ($quantity: ident, $dimension: ident, $dimensionless_const: ident) => {
        impl<const D: $dimension, S: num_traits::Float> $quantity<S, D> {
            pub fn min(self, other: Self) -> Self {
                Self(self.0.min(other.0))
            }

            pub fn max(self, other: Self) -> Self {
                Self(self.0.max(other.0))
            }
        }

        impl<const D: $dimension, S> $quantity<S, D> {
            pub fn in_units<F: num_traits::Float>(self, other: $quantity<F, D>) -> S
            where
                S: Div<F, Output = S>,
                $quantity<S, { D.dimension_div(D) }>:,
            {
                (self / other).unwrap_value()
            }
        }
    }
}

#[macro_export]
macro_rules! impl_concrete_float_methods {
    ($quantity: ident, $dimension: ident, $dimensionless_const: ident, $float_type: ident) => {
        impl<const D: $dimension> std::fmt::Debug for $quantity<$float_type, D> {
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
    }
}
