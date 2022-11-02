use std::time::Duration;

use bevy::prelude::Entity;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use raxiom::hydrodynamics::quadtree::LeafData;
use raxiom::hydrodynamics::QuadTree;
use raxiom::prelude::Extent;
use raxiom::quadtree::QuadTreeConfig;
use raxiom::units::Length;
use raxiom::units::VecLength;

fn quadtree_radius_search(quadtree: &QuadTree) {
    quadtree.get_particles_in_radius(&VecLength::meters(0.5, 0.5, 0.5), &Length::meters(0.00001));
}

fn get_quadtree(min_depth: usize) -> QuadTree {
    let min = VecLength::meters(0.0, 0.0, 0.0);
    let max = VecLength::meters(1.0, 1.0, 1.0);
    let extent = Extent::new(min, max);
    let config = QuadTreeConfig {
        min_depth,
        ..Default::default()
    };
    let tree = QuadTree::new(&config, vec![], &extent);
    let mut particles = vec![];
    tree.depth_first_map_leaf(&mut |extent: &Extent, _| particles.push(extent.center()));
    QuadTree::new(
        &config,
        particles
            .into_iter()
            .map(|pos| LeafData {
                entity: Entity::from_raw(0),
                pos,
                smoothing_length: Length::meters(0.0),
            })
            .collect(),
        &extent,
    )
}

pub fn quadtree_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadtree");
    group
        .sample_size(500)
        .measurement_time(Duration::from_secs(5));
    for depth in 1..6 {
        group.bench_with_input(
            BenchmarkId::from_parameter(depth),
            &get_quadtree(depth),
            |b, i| b.iter(|| quadtree_radius_search(i)),
        );
    }
    group.finish();
}

criterion_group!(benches, quadtree_benchmark);
criterion_main!(benches);
