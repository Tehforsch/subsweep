use std::time::Duration;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use subsweep::prelude::TwoD;
use subsweep::voronoi::Constructor;
use subsweep::voronoi::Point2d;

pub fn voronoi_benchmark(c: &mut Criterion) {
    let mut group_2d = c.benchmark_group("voronoi2d");
    group_2d
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [100, 1000, 10000] {
        group_2d.throughput(Throughput::Elements(num_particles as u64));
        group_2d.bench_function(BenchmarkId::from_parameter(num_particles), |b| {
            b.iter_batched(
                || setup_particles_2d(num_particles),
                construct_voronoi_2d,
                BatchSize::LargeInput,
            )
        });
    }
    group_2d.finish();
}

criterion_group!(benches, voronoi_benchmark);
criterion_main!(benches);

fn construct_voronoi_2d(points: Vec<Point2d>) {
    let _ = Constructor::<TwoD>::only_delaunay(points.iter());
}

fn setup_particles_2d(num_particles: usize) -> Vec<Point2d> {
    let mut rng = StdRng::seed_from_u64(1338);
    (0..num_particles)
        .map(|_| {
            let x = rng.gen_range(0.0..1.0e5);
            let y = rng.gen_range(0.0..1.0e5);
            Point2d::new(x, y)
        })
        .collect()
}
