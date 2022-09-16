mod tests {
    use bevy::prelude::App;
    use bevy::prelude::Entity;

    use super::super::QuadTree;
    use crate::domain::extent::Extent;
    use crate::domain::DomainDecompositionPlugin;
    use crate::physics::gravity::plugin::GravityPlugin;
    use crate::physics::gravity::Solver;
    use crate::quadtree;
    use crate::quadtree::*;
    use crate::units::assert_is_close;
    use crate::units::DVec2Acceleration;
    use crate::units::DVec2Length;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Mass;

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
        let acc2 = direct_sum(&solver, &pos, get_particles(n_particles).iter().collect());
        let relative_diff = (acc1 - acc2).length() / (acc1.length() + acc2.length());
        assert!(relative_diff.value() < &1e-15);
    }

    fn direct_sum(
        solver: &Solver,
        pos1: &DVec2Length,
        other_positions: Vec<&LeafData>,
    ) -> DVec2Acceleration {
        let mut total = DVec2Acceleration::zero();
        for particle in other_positions.iter() {
            total += solver.calc_gravity_acceleration(pos1, &particle.pos, particle.mass);
        }
        total
    }

    #[test]
    #[cfg(not(feature = "mpi"))]
    fn compare_parallel_quadtree_gravity_to_direct_sum() {
        use crate::communication::build_local_communication_app_with_custom_logic;
        let check = |_| {};
        build_local_communication_app_with_custom_logic(build_parallel_gravity_app, check, 2);
    }

    #[cfg(not(feature = "mpi"))]
    fn build_parallel_gravity_app(app: &mut App) {
        use crate::output;
        use crate::physics::PhysicsPlugin;
        use crate::physics::{self};

        app.insert_resource(physics::Parameters {
            opening_angle: Dimensionless::zero(),
            ..Default::default()
        })
        .insert_resource(QuadTreeConfig {
            ..Default::default()
        })
        .insert_resource(output::Parameters {
            ..Default::default()
        })
        .add_plugin(DomainDecompositionPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(GravityPlugin);
    }
}
