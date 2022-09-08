use super::mass_moments::MassMoments;
use crate::domain::quadtree::NodeDataType;
use crate::domain::AssignedSegment;
use crate::domain::Extent;

#[derive(Debug, Default)]
pub struct RemoteSegments {
    segments: Vec<AssignedSegment>,
    moments: MassMoments,
}

impl NodeDataType<Extent, AssignedSegment> for RemoteSegments {
    fn add_new_leaf_data(&mut self, extent: &Extent, new: &AssignedSegment) {
        self.moments.add_mass_at(&extent.center, &new.mass)
    }

    fn add_to_final_node(&mut self, _extent: &Extent, new: &AssignedSegment) {
        self.segments.push(new.clone())
    }
}
