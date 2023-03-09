pub const EPSILON: f64 = 1e-20;

#[derive(Copy, Clone, Debug)]
pub struct PrecisionError(f64);

pub fn is_negative(a: f64) -> Result<bool, PrecisionError> {
    if a.abs() < EPSILON {
        Err(PrecisionError(a))
    } else {
        Ok(a < 0.0)
    }
}

pub fn is_positive(a: f64) -> Result<bool, PrecisionError> {
    if a.abs() < EPSILON {
        Err(PrecisionError(a))
    } else {
        Ok(a > 0.0)
    }
}
