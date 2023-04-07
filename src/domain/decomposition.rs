use bevy::prelude::Resource;

use super::Extent;
use crate::communication::Rank;

#[derive(Resource)]
pub struct Decomposition;
impl Decomposition {
    pub(crate) fn rank_owns_part_of_search_radius(&self, _rank: Rank, _extent: Extent) -> bool {
        todo!()
    }
}
