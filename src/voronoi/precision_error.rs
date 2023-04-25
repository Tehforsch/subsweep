use num::Signed;

use crate::units::helpers::FloatError;

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
