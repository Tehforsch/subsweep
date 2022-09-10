use std::marker::PhantomData;
use std::mem::MaybeUninit;

use lazy_static::lazy_static;
use mpi::collective::SystemOperation;
use mpi::datatype::PartitionMut;
use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Communicator;
use mpi::traits::CommunicatorCollectives;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Count;
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

unsafe fn get_buffer<T>(num_elements: usize) -> Vec<T> {
    let mut buffer = Vec::with_capacity(num_elements);
    buffer.set_len(num_elements);
    buffer
}

impl<T: Equivalence> CollectiveCommunicator<T> for MpiWorld<T> {
    fn all_gather(&mut self, send: &T) -> Vec<T> {
        let count = self.world.size() as usize;
        let mut result_buffer = unsafe { get_buffer(count) };
        self.world.all_gather_into(send, &mut result_buffer[..]);
        result_buffer
    }

    fn all_gather_varcount(&mut self, send: &[T], counts: &[Count]) -> Vec<T> {
        let mut result_buffer: Vec<T> =
            unsafe { get_buffer(counts.iter().map(|x| *x as usize).sum()) };
        let displacements: Vec<Count> = counts
            .iter()
            .scan(0, |acc, &x| {
                let tmp = *acc;
                *acc += x;
                Some(tmp)
            })
            .collect();
        let mut partition = PartitionMut::new(&mut result_buffer, counts, &displacements[..]);
        self.world.all_gather_varcount_into(send, &mut partition);
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

struct UninitMsg<M>(MaybeUninit<M>);

unsafe impl<M: Equivalence> Equivalence for UninitMsg<M> {
    type Out = M::Out;

    fn equivalent_datatype() -> Self::Out {
        M::equivalent_datatype()
    }
}
