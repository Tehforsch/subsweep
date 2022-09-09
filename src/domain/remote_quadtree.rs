use super::quadtree::QuadTreeConfig;
use super::AssignedSegment;
use super::Extent;
use crate::physics::MassMoments;

type TreeData = Box<[RemoteQuadTree; 4]>;

#[derive(Debug)]
pub enum Node {
    Tree(TreeData),
    Leaf,
}

impl Node {
    fn make_node(&mut self, children: TreeData) {
        let _ = std::mem::replace(self, Node::Tree(children));
    }
}

#[derive(Debug)]
pub struct RemoteQuadTree {
    pub node: Node,
    pub data: MassMoments,
    pub particles: Vec<(Extent, AssignedSegment)>,
    pub extents: Extent,
}

impl RemoteQuadTree {
    pub fn new<'a>(
        extents: &Extent,
        config: &QuadTreeConfig,
        particles: Vec<(Extent, AssignedSegment)>,
    ) -> Self {
        let mut tree = Self::make_empty_leaf_from_extents(extents.clone());
        for (pos, data) in particles.iter() {
            tree.insert_new(config, (pos.clone(), data.clone()), 0);
        }
        tree
    }

    fn insert_new(
        &mut self,
        config: &QuadTreeConfig,
        data: (Extent, AssignedSegment),
        depth: usize,
    ) {
        self.data
            .add_mass_at(&data.1.mass.center_of_mass(), &data.1.mass.total());
        self.insert(config, data, depth)
    }

    fn insert(&mut self, config: &QuadTreeConfig, data: (Extent, AssignedSegment), depth: usize) {
        if let Node::Leaf = self.node {
            if self.particles.is_empty() || depth == config.max_depth {
                self.particles.push((data.0, data.1));
                return;
            } else {
                self.subdivide(config, depth);
            }
        }
        if let Node::Tree(ref mut children) = self.node {
            let index = data.0.get_quadrant_index_for_extent(&self.extents);
            match index {
                Some(index) => {
                    let quadrant = &mut children[index];
                    quadrant.insert_new(&config, data, depth + 1);
                }
                None => self.particles.push((data.0, data.1)),
            }
        }
    }

    fn subdivide(&mut self, config: &QuadTreeConfig, depth: usize) {
        debug_assert!(matches!(self.node, Node::Leaf));
        let quadrants = self.extents.get_quadrants();
        let children = Box::new(quadrants.map(Self::make_empty_leaf_from_extents));
        self.node.make_node(children);
        for particle in self.particles.drain(..).collect::<Vec<_>>() {
            self.insert(config, particle, depth);
        }
    }

    fn make_empty_leaf_from_extents(extents: Extent) -> Self {
        Self {
            node: Node::Leaf,
            data: MassMoments::default(),
            particles: vec![],
            extents,
        }
    }

    pub fn depth_first_map_node(
        &self,
        closure: &mut impl FnMut(&Extent, &[(Extent, AssignedSegment)], &MassMoments) -> (),
    ) {
        closure(&self.extents, &self.particles, &self.data);
        match self.node {
            Node::Tree(ref node) => {
                for child in node.iter() {
                    child.depth_first_map_node(closure);
                }
            }
            _ => {}
        }
    }
}
