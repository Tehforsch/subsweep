use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use raxiom::domain::LeafData;
use raxiom::prelude::Extent;
use raxiom::prelude::ParticleId;
use raxiom::prelude::SimulationBox;
use raxiom::quadtree::NodeDataType;
use raxiom::quadtree::QuadTree;
use raxiom::quadtree::QuadTreeConfig;
use raxiom::units::Length;
use raxiom::units::VecLength;

#[derive(Default)]
struct Empty;

impl<T> NodeDataType<T> for Empty {
    fn update_with(&mut self, _: &T) {}
}

fn quadtree_radius_search(quadtree: &QuadTree<Empty, LeafData>) {
    let box_size = SimulationBox::new(quadtree.extent.clone());
    for _ in quadtree.iter_particles_in_radius(
        &box_size,
        VecLength::meters(0.5, 0.5, 0.5),
        Length::meters(0.01),
    ) {}
}

fn get_quadtree(num_parts: usize) -> QuadTree<Empty, LeafData> {
    let num_parts_per_dimension = (num_parts as f64).cbrt().floor() as usize;
    let mut particles = vec![];
    let get_p = |x, y, z| {
        let x = x as f64 / num_parts_per_dimension as f64;
        let y = y as f64 / num_parts_per_dimension as f64;
        let z = z as f64 / num_parts_per_dimension as f64;
        VecLength::meters(x, y, z)
    };
    for x in 0..num_parts_per_dimension {
        for y in 0..num_parts_per_dimension {
            for z in 0..num_parts_per_dimension {
                particles.push(LeafData {
                    pos: get_p(x, y, z),
                    id: ParticleId(0),
                });
            }
        }
    }
    let extent = Extent::from_positions(particles.iter().map(|x| &x.pos)).unwrap();
    let config = QuadTreeConfig::default();
    QuadTree::new(&config, particles, &extent)
}

pub fn quadtree_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadtree");
    group.noise_threshold(0.05);
    for num_particles in [1000, 10000, 100000, 1000000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_particles),
            &get_quadtree(num_particles),
            |b, i| b.iter(|| quadtree_radius_search(i)),
        );
    }
    group.finish();
}

criterion_group!(benches, quadtree_benchmark);
criterion_main!(benches);
