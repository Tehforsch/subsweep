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
use raxiom::voronoi::DelaunayTriangulation;
use raxiom::voronoi::Point2d;
use raxiom::voronoi::Point3d;
use raxiom::voronoi::ThreeD;
use raxiom::voronoi::TwoD;
use raxiom::voronoi::VoronoiGrid;

pub fn voronoi_benchmark(c: &mut Criterion) {
    let mut group_2d = c.benchmark_group("voronoi2d");
    group_2d
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [100, 1000] {
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
    let mut group_3d = c.benchmark_group("voronoi3d");
    group_3d
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [100, 1000] {
        group_3d.throughput(Throughput::Elements(num_particles as u64));
        group_3d.bench_function(BenchmarkId::from_parameter(num_particles), |b| {
            b.iter_batched(
                || setup_particles_3d(num_particles),
                construct_voronoi_3d,
                BatchSize::LargeInput,
            )
        });
    }
    group_3d.finish();
}

criterion_group!(benches, voronoi_benchmark);
criterion_main!(benches);

fn construct_voronoi_2d(points: Vec<Point2d>) {
    let t = DelaunayTriangulation::<TwoD>::construct_no_key(&points);
    let _: VoronoiGrid<TwoD> = (&t).into();
}

fn construct_voronoi_3d(points: Vec<Point3d>) {
    let t = DelaunayTriangulation::<ThreeD>::construct_no_key(&points);
    let _: VoronoiGrid<ThreeD> = (&t).into();
}

fn setup_particles_3d(num_particles: usize) -> Vec<Point3d> {
    let mut rng = StdRng::seed_from_u64(1338);
    (0..num_particles)
        .map(|_| {
            let x = rng.gen_range(0.0..1.0e5);
            let y = rng.gen_range(0.0..1.0e5);
            let z = rng.gen_range(0.0..1.0e5);
            Point3d::new(x, y, z)
        })
        .collect()
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
