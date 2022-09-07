use crate::domain::Extent;
use crate::units::VecLength;

pub trait InsertionData {
    fn get_quadrant_index(&self, extent: &Extent) -> usize;
}

impl InsertionData for VecLength {
    fn get_quadrant_index(&self, extent: &Extent) -> usize {
        extent.get_quadrant_index(self)
    }
}
