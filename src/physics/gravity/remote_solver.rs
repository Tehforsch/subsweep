use bevy::prelude::Commands;
use bevy::prelude::Res;

use super::remote_segment_data::RemoteSegments;
use crate::domain::quadtree::insertion_data::InsertionData;
use crate::domain::quadtree::QuadTreeConfig;
use crate::domain::quadtree::{self};
use crate::domain::AssignedSegment;
use crate::domain::Extent;
use crate::domain::GlobalExtent;
use crate::domain::Segments;

pub type RemoteQuadTree = quadtree::QuadTree<RemoteSegments, Extent, AssignedSegment>;

pub fn construct_remote_quad_tree_system(
    mut commands: Commands,
    config: Res<QuadTreeConfig>,
    segments: Res<Segments>,
    extent: Res<GlobalExtent>,
) {
    let data = segments
        .0
        .iter()
        .filter_map(|segment| {
            segment
                .extent
                .clone()
                .map(|extent| (extent, segment.clone()))
        })
        .collect();
    let quadtree = RemoteQuadTree::new(&extent.0, &config, data);
    commands.insert_resource(quadtree);
}

impl InsertionData for Extent {
    fn get_quadrant_index(&self, extent: &Extent) -> Option<usize> {
        self.get_quadrants()
            .iter()
            .enumerate()
            .find(|(_, quad)| quad.encompasses(extent))
            .map(|(i, _)| i)
    }
}
