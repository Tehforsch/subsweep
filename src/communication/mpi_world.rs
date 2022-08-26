use std::marker::PhantomData;

use lazy_static::lazy_static;
use mpi::collective::SystemOperation;
use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Communicator;
use mpi::traits::CommunicatorCollectives;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Tag;
use mpi::Threading;

use super::collective_communicator::SumCommunicator;
use super::world_communicator::WorldCommunicator;
use super::CollectiveCommunicator;
use super::Identified;
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
    tag: Tag,
}

impl<T> MpiWorld<T> {
    pub fn new(tag: Tag) -> Self {
        let world = MPI_UNIVERSE.world();
        Self {
            world,
            tag,
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
        process.send_with_tag(&num, self.tag);
        if num > 0 {
            process.send_with_tag(&data, self.tag);
        }
    }

    fn receive_vec(&mut self, rank: Rank) -> Vec<S> {
        let process = self.world.process_at_rank(rank);
        let (num_received, _): (usize, Status) = process.receive_with_tag(self.tag);
        if num_received > 0 {
            let (data, _) = process.receive_vec_with_tag(self.tag);
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

impl<T: Equivalence + Clone> CollectiveCommunicator<T> for MpiWorld<T> {
    fn all_gather(&mut self, send: &T) -> Vec<T> {
        let count = self.world.size() as usize;
        // we can replace this by MaybeUninit at some point, but that will require unsafe
        let mut result_buffer = vec![send.clone(); count];
        self.world.all_gather_into(send, &mut result_buffer[..]);
        result_buffer
    }
}

impl<T: Equivalence + Clone> SumCommunicator<T> for MpiWorld<T> {
    fn collective_sum(&mut self, send: &T) -> T {
        let mut result = send.clone();
        self.world
            .all_reduce_into(send, &mut result, SystemOperation::sum());
        result
    }
}

impl<T> From<MpiWorld<T>> for MpiWorld<Identified<T>> {
    fn from(other: MpiWorld<T>) -> Self {
        Self {
            world: other.world,
            marker: PhantomData::default(),
            tag: other.tag,
        }
    }
}
