use hdf5::types::FloatSize;
use hdf5::types::TypeDescriptor;
use hdf5::H5Type;

use super::dimension::Dimension;
use super::quantity::Quantity;

unsafe impl<const D: Dimension> H5Type for Quantity<f64, D> {
    fn type_descriptor() -> hdf5::types::TypeDescriptor {
        TypeDescriptor::Float(FloatSize::U8)
    }
}

unsafe impl<const D: Dimension> H5Type for Quantity<glam::DVec2, D> {
    fn type_descriptor() -> hdf5::types::TypeDescriptor {
        TypeDescriptor::FixedArray(Box::new(TypeDescriptor::Float(FloatSize::U4)), 2)
    }
}
