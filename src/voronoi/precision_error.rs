use num::BigRational;
use num::Signed;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PrecisionError;

impl PrecisionError {
    pub fn check<F: Signed + FloatError>(a: &F) -> Result<(), PrecisionError> {
        if FloatError::is_too_close_to_zero(a) {
            Err(PrecisionError)
        } else {
            Ok(())
        }
    }
}

pub const ERROR_TRESHOLD: f64 = 1e-9;

pub trait FloatError {
    fn is_too_close_to_zero(&self) -> bool;
}

impl FloatError for f64 {
    fn is_too_close_to_zero(&self) -> bool {
        self.abs() < ERROR_TRESHOLD
    }
}

impl FloatError for BigRational {
    fn is_too_close_to_zero(&self) -> bool {
        false
    }
}
