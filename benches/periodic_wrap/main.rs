use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;
use raxiom::prelude::Extent;
use raxiom::prelude::Float;
use raxiom::prelude::SimulationBox;
use raxiom::units::VecLength;

const NUM_PARTICLES: usize = 1000;

fn get_box_and_particles() -> (SimulationBox, Vec<VecLength>) {
    let min = VecLength::meters(0.0, 0.0, 0.0);
    let max = VecLength::meters(1.0, 1.0, 1.0);
    let extent = Extent::new(min, max);
    let positions: Vec<_> = (0..NUM_PARTICLES)
        .map(|i| {
            let f = i as Float;
            0.001 * VecLength::meters(f, f.powi(2), f.powi(3))
        })
        .collect();
    (extent.into(), positions)
}

pub fn periodic_wrap_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("periodic_wrap");
    group.noise_threshold(0.05);
    group.throughput(Throughput::Elements(NUM_PARTICLES as u64));
    group.bench_with_input(
        "periodic_wrap",
        &get_box_and_particles(),
        |b, (box_, particles)| {
            b.iter(|| {
                for pos in particles.iter() {
                    box_.periodic_wrap(*pos);
                }
            })
        },
    );
    group.bench_with_input(
        "periodic_distance_vec",
        &get_box_and_particles(),
        |b, (box_, particles)| {
            b.iter(|| {
                for chunk in particles.chunks_exact(2) {
                    box_.periodic_distance_vec(&chunk[0], &chunk[1]);
                }
            })
        },
    );
    group.finish();
}

criterion_group!(benches, periodic_wrap_benchmark);
criterion_main!(benches);
