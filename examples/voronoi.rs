use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use raxiom::voronoi::DelaunayTriangulation;
use raxiom::voronoi::Point2d;
use raxiom::voronoi::TwoD;
use raxiom::voronoi::VoronoiGrid;

pub fn main() {
    let p = setup_particles_2d(20000);
    construct_voronoi_2d(p);
}

fn construct_voronoi_2d(points: Vec<Point2d>) {
    let t = DelaunayTriangulation::<TwoD>::construct_no_key(&points);
    let _: VoronoiGrid<TwoD> = (&t).into();
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
