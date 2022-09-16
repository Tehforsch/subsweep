#[cfg(not(feature = "mpi"))]
mod parallel;

mod tests {
    use bevy::prelude::Entity;

    use super::super::QuadTree;
    use crate::domain::extent::Extent;
    use crate::physics::gravity::Solver;
    use crate::quadtree;
    use crate::quadtree::*;
    use crate::units::assert_is_close;
    use crate::units::Acceleration;
    use crate::units::DVec2Acceleration;
    use crate::units::DVec2Length;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Mass;
    use crate::units::VecAcceleration;

    fn get_particles(n: i32) -> Vec<LeafData> {
        (1..n)
            .flat_map(move |x| {
                (1..n).map(move |y| LeafData {
                    entity: Entity::from_raw((x * n + y) as u32),
                    pos: DVec2Length::meter(x as f64, y as f64),
                    mass: Mass::kilogram(x as f64 * y as f64),
                })
            })
            .collect()
    }

    fn get_tree_for_particles(n: i32) -> QuadTree {
        let particles = get_particles(n);
        let extent = Extent::from_positions(particles.iter().map(|part| &part.pos)).unwrap();
        QuadTree::new(&QuadTreeConfig::default(), particles, &extent)
    }

    #[test]
    fn mass_sum() {
        let tree = get_tree_for_particles(7);
        check_all_sub_trees(&tree);
    }

    fn check_all_sub_trees(tree: &QuadTree) {
        check_mass(tree);
        match tree.node {
            quadtree::Node::Tree(ref children) => {
                for child in children.iter() {
                    check_all_sub_trees(child)
                }
            }
            quadtree::Node::Leaf(_) => {}
        }
    }

    fn check_mass(tree: &QuadTree) {
        let mut total = Mass::zero();
        tree.depth_first_map_leaf(&mut |_, data| total += data.iter().map(|p| p.mass).sum());
        assert_is_close(tree.data.moments.total(), total);
    }

    #[test]
    fn compare_quadtree_gravity_to_direct_sum() {
        let n_particles = 50;
        let tree = get_tree_for_particles(n_particles);
        let pos = DVec2Length::meter(3.5, 3.5);
        let solver = Solver {
            opening_angle: Dimensionless::zero(),
            softening_length: Length::zero(),
        };
        let acc1 = solver.traverse_tree(&tree, &pos);
        let acc2 = direct_sum(
            &solver,
            &pos,
            get_particles(n_particles)
                .iter()
                .map(|part| (part.pos, part.mass))
                .collect(),
        );
        compare_accelerations(acc1, acc2);
    }

    pub(super) fn compare_accelerations(acc1: VecAcceleration, acc2: VecAcceleration) {
        let min_acc = Acceleration::meters_per_second_squared(1e-15);
        let relative_diff = (acc1 - acc2).length() / (acc1.length() + acc2.length() + min_acc);
        assert!(relative_diff.value() < &1e-15);
    }

    pub(super) fn direct_sum(
        solver: &Solver,
        pos1: &DVec2Length,
        other_positions: Vec<(DVec2Length, Mass)>,
    ) -> DVec2Acceleration {
        let mut total = DVec2Acceleration::zero();
        for (pos, mass) in other_positions.into_iter() {
            total += solver.calc_gravity_acceleration(pos1, &pos, mass);
        }
        total
    }
}
