mod constants;
mod deserialize;
mod dimension;
mod mpi;
mod quantities_and_units;
mod quantity;

pub use constants::*;
pub use quantities_and_units::*;

#[cfg(test)]
mod tests {
    use super::dimension::Dimension;
    use super::quantity::Quantity;
    use super::Dimensionless;
    use super::Length;

    pub(super) fn assert_is_close<const U: Dimension>(x: Quantity<f32, U>, y: Quantity<f32, U>) {
        const EPSILON: f32 = 1e-20;
        assert!(
            (x - y).abs().unwrap_value() < EPSILON,
            "{} {}",
            x.unwrap_value(),
            y.unwrap_value()
        )
    }

    #[test]
    fn add_same_unit() {
        let x = Length::meter(1.0);
        let y = Length::meter(10.0);
        assert_is_close(x + y, Length::meter(11.0));
    }

    #[test]
    fn add_different_units() {
        let x = Length::meter(1.0);
        let y = Length::kilometer(10.0);
        assert_is_close(x + y, Length::meter(10001.0));
    }

    #[test]
    fn sub_different_units() {
        let x = Length::meter(1.0);
        let y = Length::kilometer(10.0);
        assert_is_close(x - y, Length::meter(-9999.0));
    }

    #[test]
    fn div_same_unit() {
        let x = Length::meter(1.0);
        let y = Length::meter(10.0);
        assert_is_close(x / y, Dimensionless::dimensionless(0.1));
    }
}
