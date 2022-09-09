use bevy::prelude::Commands;
use bevy::prelude::Res;

use super::remote_segment_data::RemoteSegments;
use crate::communication::WorldRank;
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
    rank: Res<WorldRank>,
) {
    let data = segments
        .0
        .iter()
        .filter_map(|segment| {
            if segment.rank == rank.0 {
                return None;
            }
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
        extent
            .get_quadrants()
            .iter()
            .enumerate()
            .find(|(_, quad)| quad.encompasses(self))
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteQuadTree;
    use crate::domain::quadtree;
    use crate::domain::quadtree::QuadTreeConfig;
    use crate::domain::AssignedSegment;
    use crate::domain::Extent;
    use crate::domain::Segment;
    use crate::physics::MassMoments;
    use crate::units::assert_is_close;
    use crate::units::Length;
    use crate::units::Mass;
    use crate::units::VecLength;

    fn get_mass_at(mass: f32, pos: VecLength) -> MassMoments {
        let mut mass_moments = MassMoments::default();
        mass_moments.add_mass_at(&pos, &Mass::kilogram(mass));
        mass_moments
    }

    fn get_extent(x1: f32, y1: f32, x2: f32, y2: f32) -> Extent {
        Extent::new(
            Length::meter(x1),
            Length::meter(x2),
            Length::meter(y1),
            Length::meter(y2),
        )
    }

    fn get_assigned_segments(n: usize) -> Vec<AssignedSegment> {
        (0..n)
            .map(|i| {
                let extent = get_extent(
                    i as f32 / n as f32,
                    i as f32 / n as f32,
                    (i + 1) as f32 / n as f32,
                    (i + 1) as f32 / n as f32,
                );
                let mass = get_mass_at(1.0, extent.center());
                AssignedSegment {
                    segment: Segment::from_num(i as u64, i as u64 + 1, 1),
                    rank: 1,
                    extent: Some(extent),
                    mass,
                }
            })
            .collect()
    }

    fn check_mass(tree: &RemoteQuadTree) {
        let mut total = Mass::zero();
        tree.depth_first_map(&mut |_, data| {
            total += data.iter().map(|(_, p)| p.mass.total()).sum()
        });
        assert_is_close(tree.data.moments.total(), total);
    }

    fn check_mass_of_all_sub_trees(tree: &RemoteQuadTree) {
        check_mass(tree);
        match tree.node {
            quadtree::Node::Tree(ref children) => {
                for child in children.iter() {
                    check_mass_of_all_sub_trees(child)
                }
            }
            quadtree::Node::Leaf(_) => {}
        }
    }

    #[test]
    fn remote_quadtree_counts_mass_correctly() {
        let segments = get_assigned_segments(3);
        let data = segments
            .iter()
            .filter_map(|segment| {
                segment
                    .extent
                    .clone()
                    .map(|extent| (extent, segment.clone()))
            })
            .collect();
        let extent =
            Extent::get_all_encompassing(segments.iter().filter_map(|seg| seg.extent.as_ref()))
                .unwrap();
        let config = QuadTreeConfig { max_depth: 10 };

        let quadtree = RemoteQuadTree::new(&extent, &config, data);
        dbg!(&quadtree);
        check_mass_of_all_sub_trees(&quadtree);
    }
}
