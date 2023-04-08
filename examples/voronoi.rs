use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use raxiom::prelude::ParticleId;
use raxiom::prelude::ThreeD;
use raxiom::voronoi::Constructor;
use raxiom::voronoi::Point3d;

pub fn main() {
    let p = setup_particles_3d(100000);
    construct_voronoi_3d(p);
}

fn construct_voronoi_3d(points: Vec<Point3d>) {
    let _ = Constructor::<ThreeD>::new(
        points
            .iter()
            .enumerate()
            .map(|(i, p)| (ParticleId(i as u64), *p)),
    )
    .voronoi();
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
