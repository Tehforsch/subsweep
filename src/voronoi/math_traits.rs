use num::traits::NumOps;
use num::Signed;

use super::precision_error::FloatError;

pub trait Num:
    num::Num + Clone + Signed + PartialOrd + FloatError + std::fmt::Debug + NumOps
{
}

impl<T> Num for T where T: num::Num + Clone + Signed + PartialOrd + FloatError + std::fmt::Debug {}
