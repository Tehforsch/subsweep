use std::ops::Deref;

use hdf5::H5Type;

use crate::named::Named;
use crate::units::Dimension;
use crate::units::Quantity;

pub trait ToDataset: H5Type + Named {
    fn dimension() -> Dimension;
    fn convert_base_units(self, factor: f64) -> Self;
}

impl<const D: Dimension, S, T> ToDataset for T
where
    S: Clone + 'static + std::ops::Mul<f64, Output = S>,
    T: Named + H5Type + Deref<Target = Quantity<S, D>> + From<Quantity<S, D>>,
{
    fn dimension() -> Dimension {
        D
    }

    fn convert_base_units(self, factor: f64) -> T {
        (T::deref(&self).clone() * factor).into()
    }
}
