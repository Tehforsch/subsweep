mod dimension;
mod quantities_and_units;
mod quantity;

pub use quantities_and_units::*;

#[cfg(test)]
mod tests {
    use super::dimension::Dimension;
    use super::meter;
    use super::quantity::Quantity;

    fn assert_is_close<const U: Dimension>(x: Quantity<U>, y: Quantity<U>) {
        const EPSILON: f64 = 1e-10;
        assert!((x - y).abs().unwrap_value() < EPSILON)
    }

    #[test]
    fn add_units() {
        let x = meter(1.0);
        let y = meter(10.0);
        assert_is_close(x + y, meter(11.0));
    }
}
