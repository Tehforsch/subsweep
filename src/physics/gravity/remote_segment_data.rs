use super::mass_moments::MassMoments;
use crate::domain::quadtree::NodeDataType;
use crate::domain::AssignedSegment;

#[derive(Debug, Default)]
pub struct RemoteSegmentData {
    segments: Vec<AssignedSegment>,
    moments: MassMoments,
}

impl<P, L> NodeDataType<P, L> for RemoteSegmentData {
    fn add_new_leaf_data(&mut self, _p: &P, _l: &L) {}
}
