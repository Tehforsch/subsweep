use std::marker::PhantomData;

use lazy_static::lazy_static;
use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Communicator;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Threading;

use super::world_communicator::WorldCommunicator;
use super::SizedCommunicator;

lazy_static! {
    pub static ref MPI_UNIVERSE: Universe = {
        let (universe, _threading) = mpi::initialize_with_threading(Threading::Multiple).unwrap();
        universe
    };
}

#[derive(Clone)]
pub struct MpiWorld<T> {
    world: SystemCommunicator,
    marker: PhantomData<T>,
}

impl<T> MpiWorld<T> {
    pub fn new() -> Self {
        let world = MPI_UNIVERSE.world();
        Self {
            world,
            marker: PhantomData::default(),
        }
    }
}

impl<S, T> WorldCommunicator<S> for MpiWorld<T>
where
    S: Equivalence,
{
    fn send_vec(&mut self, rank: Rank, data: Vec<S>) {
        let num = data.len();
        let process = self.world.process_at_rank(rank);
        process.send(&num);
        for d in data.into_iter() {
            process.send(&d);
        }
    }

    fn receive_vec(&mut self, rank: Rank) -> Vec<S> {
        let process = self.world.process_at_rank(rank);
        let (num_received, _): (usize, Status) = process.receive();
        if num_received > 0 {
            let (data, _) = process.receive_vec();
            return data;
        }
        vec![]
    }
}

impl<T> SizedCommunicator for MpiWorld<T> {
    fn rank(&self) -> i32 {
        self.world.rank()
    }

    fn size(&self) -> usize {
        self.world.size() as usize
    }
}
