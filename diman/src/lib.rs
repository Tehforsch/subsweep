#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
#![feature(const_fn_floating_point_arithmetic)]

mod dimension;
mod quantities_and_units;
mod quantity;
mod sampling;

#[cfg(feature = "hdf5")]
mod hdf5;

#[cfg(feature = "mpi")]
mod mpi;

#[cfg(feature = "serde")]
mod deserialize;

pub use dimension::Dimension;
pub use quantities_and_units::*;
pub use quantity::Quantity;
#[cfg(test)]
pub use tests::assert_is_close;

#[cfg(test)]
mod tests {
    use super::dimension::Dimension;
    use super::quantity::Quantity;
    use super::Dimensionless;
    use super::Length;

    pub(crate) fn assert_is_close<const U: Dimension>(x: Quantity<f64, U>, y: Quantity<f64, U>) {
        const EPSILON: f64 = 1e-20;
        assert!(
            (x - y).abs().unwrap_value() < EPSILON,
            "{} {}",
            x.unwrap_value(),
            y.unwrap_value()
        )
    }

    #[test]
    fn add_same_unit() {
        let x = Length::meters(1.0);
        let y = Length::meters(10.0);
        assert_is_close(x + y, Length::meters(11.0));
    }

    #[test]
    fn add_different_units() {
        let x = Length::meters(1.0);
        let y = Length::kilometers(10.0);
        assert_is_close(x + y, Length::meters(10001.0));
    }

    #[test]
    fn sub_different_units() {
        let x = Length::meters(1.0);
        let y = Length::kilometers(10.0);
        assert_is_close(x - y, Length::meters(-9999.0));
    }

    #[test]
    fn div_same_unit() {
        let x = Length::meters(1.0);
        let y = Length::meters(10.0);
        assert_is_close(x / y, Dimensionless::dimensionless(0.1));
    }
}
