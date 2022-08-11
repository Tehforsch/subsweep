mod dimension;
mod mpi;
mod quantities_and_units;
mod quantity;

pub use quantities_and_units::*;

#[cfg(test)]
mod tests {
    use super::dimension::Dimension;
    use super::f64::dimensionless;
    use super::f64::kilometer;
    use super::f64::meter;
    use super::quantity::Quantity;

    fn assert_is_close<const U: Dimension>(x: Quantity<f64, U>, y: Quantity<f64, U>) {
        const EPSILON: f64 = 1e-20;
        assert!(*(x - y).abs().unwrap_value() < EPSILON)
    }

    #[test]
    fn add_same_unit() {
        let x = meter(1.0);
        let y = meter(10.0);
        assert_is_close(x + y, meter(11.0));
    }

    #[test]
    fn add_different_units() {
        let x = meter(1.0);
        let y = kilometer(10.0);
        assert_is_close(x + y, meter(10001.0));
    }

    #[test]
    fn sub_different_units() {
        let x = meter(1.0);
        let y = kilometer(10.0);
        assert_is_close(x - y, meter(-9999.0));
    }

    #[test]
    fn div_same_unit() {
        let x = meter(1.0);
        let y = meter(10.0);
        assert_is_close(x / y, dimensionless(0.1));
    }
}
