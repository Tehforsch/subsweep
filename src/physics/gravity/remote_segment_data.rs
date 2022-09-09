use super::mass_moments::MassMoments;
use crate::domain::quadtree::NodeDataType;
use crate::domain::AssignedSegment;
use crate::domain::Extent;

#[derive(Debug, Default)]
pub struct RemoteSegments {
    pub segments: Vec<AssignedSegment>,
    pub moments: MassMoments,
}

impl NodeDataType<Extent, AssignedSegment> for RemoteSegments {
    fn handle_insertion(&mut self, _extent: &Extent, new: &AssignedSegment) {
        self.moments
            .add_mass_at(&new.mass.center_of_mass(), &new.mass.total())
    }

    fn add_to_final_node(&mut self, _extent: &Extent, new: &AssignedSegment) {
        self.segments.push(new.clone())
    }
}
