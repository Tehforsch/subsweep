use std::time::Instant;

use raxiom::communication::DataByRank;
use raxiom::communication::ExchangeCommunicator;
use raxiom::communication::MpiWorld;
use raxiom::communication::SizedCommunicator;
use raxiom::communication::MPI_UNIVERSE;
use raxiom::units::Time;

fn exchange_all(num_elements: usize, world_size: usize, this_rank: usize) {
    let data: DataByRank<Vec<usize>> = (0..world_size)
        .filter_map(|rank| {
            Some((rank as i32, (0..num_elements).collect()))
                .filter(|(rank, _)| *rank != this_rank as i32)
        })
        .collect();
    let mut exch = ExchangeCommunicator::new();
    let out = exch.exchange_all(data);
    assert_eq!(out.size(), (world_size - 1) * num_elements);
}

fn all_reduce_sum(world_size: usize, rank: usize) {
    let mut world = MpiWorld::<usize>::new();
    let sum = world.all_reduce_sum(&(rank as u64));
    assert_eq!(sum as usize, (world_size - 1) * world_size / 2);
}

fn run_timing(name: &str, f: impl Fn() -> (), num_iterations: usize) {
    let rank = MpiWorld::<usize>::new().rank() as usize;
    let start = Instant::now();
    let time_elapsed_micros =
        || Time::microseconds(Instant::now().duration_since(start).as_micros() as f64);
    for _ in 0..num_iterations {
        f();
    }
    if rank == 0 {
        println!(
            "{:<22}:  {:>10.04} ms / it",
            name,
            (time_elapsed_micros() / num_iterations as f64).in_milliseconds()
        );
    }
}

pub fn main() {
    let rank = MpiWorld::<usize>::new().rank() as usize;
    let size = MpiWorld::<usize>::new().size() as usize;
    run_timing("all_reduce_sum", || all_reduce_sum(size, rank), 1000000);
    for num_elements in [1000, 10000, 100000, 1000000] {
        let num_iterations = 10000000 / num_elements;
        run_timing(
            &format!("exchange_all {:<8}", num_elements),
            || exchange_all(num_elements, size, rank),
            num_iterations,
        );
    }
    MPI_UNIVERSE.drop()
}
