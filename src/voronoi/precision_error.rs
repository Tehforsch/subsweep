pub const EPSILON: f64 = 1e-10;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PrecisionError;

impl PrecisionError {
    pub fn check(a: f64) -> Result<(), PrecisionError> {
        if a.abs() < EPSILON {
            Err(PrecisionError)
        } else {
            Ok(())
        }
    }
}

pub fn is_negative(a: f64) -> Result<bool, PrecisionError> {
    PrecisionError::check(a).map(|_| a < 0.0)
}

pub fn is_positive(a: f64) -> Result<bool, PrecisionError> {
    PrecisionError::check(a).map(|_| a > 0.0)
}
