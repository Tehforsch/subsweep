use crate::domain::Extent;
use crate::units::VecLength;

pub trait InsertionData {
    fn get_quadrant_index(&self, extent: &Extent) -> Option<usize>;
}

impl InsertionData for VecLength {
    fn get_quadrant_index(&self, extent: &Extent) -> Option<usize> {
        Some(extent.get_quadrant_index(self))
    }
}
