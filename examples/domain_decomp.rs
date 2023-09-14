use std::time::Instant;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use raxiom::communication::MpiWorld;
use raxiom::communication::SizedCommunicator;
use raxiom::communication::MPI_UNIVERSE;
use raxiom::domain::get_decomposition_from_points_and_box;
use raxiom::domain::DecompositionState;
use raxiom::prelude::*;
use raxiom::units::Time;
use raxiom::units::VecLength;

fn setup_points_3d(num_particles: usize) -> Vec<VecLength> {
    let mut rng = StdRng::seed_from_u64(1338);
    (0..num_particles)
        .map(|_| {
            let x = rng.gen_range(0.0..1.0e5);
            let y = rng.gen_range(0.0..1.0e5);
            let z = rng.gen_range(0.0..1.0e5);
            VecLength::meters(x, y, z)
        })
        .collect()
}

fn run_res(res: usize, world_size: usize) -> DecompositionState {
    let points = setup_points_3d(res * res * res / world_size);
    let box_ = SimulationBox(Extent::from_positions(points.iter()).unwrap());
    get_decomposition_from_points_and_box(points, &box_, world_size)
}

fn main() {
    let rank = MpiWorld::<usize>::new().rank();
    let size = MpiWorld::<usize>::new().size() as usize;
    let mut start = Instant::now();
    let mut time_elapsed = || {
        let time = Time::microseconds(Instant::now().duration_since(start).as_micros() as f64);
        start = Instant::now();
        time
    };
    for res in [270, 540, 1080] {
        let decomp = run_res(res, size);
        let time = time_elapsed();
        if rank == 0 {
            println!(
                "{}: {} s (imbalance: {})",
                res,
                time.in_seconds(),
                decomp.get_imbalance()
            );
        }
    }
    MPI_UNIVERSE.drop()
}
