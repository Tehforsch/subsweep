use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::Mutex;

use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use lazy_static::lazy_static;
use mpi::collective::SystemOperation;
use mpi::datatype::PartitionMut;
use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::request::Scope;
use mpi::request::WaitGuard;
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
use super::DataByRank;
use super::Identified;
use super::SizedCommunicator;

/// A wrapper around universe which contains the universe in an
/// Option. This allows calling .take at program completion so that
/// the Universe is dropped which will call MPI_FINALIZE.  This is
/// necessary because anything in a lazy_static will never be dropped.
#[derive(Deref, DerefMut)]
pub struct StaticUniverse(Arc<Mutex<Option<Universe>>>);

impl StaticUniverse {
    pub fn world(&self) -> SystemCommunicator {
        self.0.lock().unwrap().as_ref().unwrap().world()
    }

    pub fn drop(&self) {
        let _ = self.0.lock().unwrap().take();
    }
}

lazy_static! {
    pub static ref MPI_UNIVERSE: StaticUniverse = {
        let threading = Threading::Multiple;
        let (universe, threading_initialized) = mpi::initialize_with_threading(threading).unwrap();
        assert_eq!(
            threading, threading_initialized,
            "Could not initialize MPI with Multithreading"
        );
        StaticUniverse(Arc::new(Mutex::new(Some(universe))))
    };
}

#[derive(Clone)]
pub struct MpiWorld<T> {
    world: SystemCommunicator,
    _marker: PhantomData<T>,
    tag: Tag,
}

impl<T> MpiWorld<T> {
    pub fn new(tag: Tag) -> Self {
        let world = MPI_UNIVERSE.world();
        Self {
            world,
            tag,
            _marker: PhantomData::default(),
        }
    }

    pub fn world(&self) -> &SystemCommunicator {
        &self.world
    }
}

impl<S, T> WorldCommunicator<S> for MpiWorld<T>
where
    S: Equivalence,
{
    fn blocking_send_vec(&mut self, rank: Rank, data: &[S]) {
        let num = data.len();
        let process = self.world.process_at_rank(rank);
        process.send_with_tag(&num, self.tag);
        if num > 0 {
            process.send_with_tag(data, self.tag);
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

    #[must_use]
    fn immediate_send_vec<'a, Sc: Scope<'a>>(
        &mut self,
        scope: Sc,
        rank: Rank,
        data: &'a [S],
    ) -> Option<WaitGuard<'a, [S], Sc>> {
        let num = data.len();
        let process = self.world.process_at_rank(rank);
        process.buffered_send_with_tag(&num, self.tag);
        if num > 0 {
            Some(WaitGuard::from(
                process.immediate_send_with_tag(scope, data, self.tag),
            ))
        } else {
            None
        }
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
    let mut buffer: Vec<MaybeUninit<T>> = Vec::with_capacity(num_elements);
    unsafe {
        buffer.set_len(num_elements);
        mem::transmute(buffer)
    }
}

impl<T: Equivalence> CollectiveCommunicator<T> for MpiWorld<T> {
    fn all_gather(&mut self, send: &T) -> Vec<T> {
        let world_size = self.world.size() as usize;
        let mut result_buffer = unsafe { get_buffer(world_size) };
        self.world.all_gather_into(send, &mut result_buffer[..]);
        result_buffer
    }

    fn all_gather_vec(&mut self, send: &[T]) -> DataByRank<Vec<T>> {
        let world_size = self.world.size() as usize;
        let num_elements = send.len();
        let mut result_buffer = unsafe { get_buffer::<T>(world_size * num_elements) };
        self.world.all_gather_into(send, &mut result_buffer[..]);
        let mut data = DataByRank::empty();
        for i in 0..world_size {
            data.insert(i as Rank, result_buffer.drain(0..num_elements).collect())
        }
        data
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
            _marker: PhantomData::default(),
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

#[cfg(test)]
mod tests {
    use mpi::request::scope;
    use mpi::Tag;

    use super::MpiWorld;
    use crate::communication::WorldCommunicator;

    #[test]
    fn immediate_send_receive() {
        let mut world = MpiWorld::<i32>::new(Tag::default());
        let x: [i32; 3] = [1, 2, 3];
        let result: Vec<i32> = scope(|scope| {
            let _guard = world.immediate_send_vec(scope, 0, &x);
            world.receive_vec(0)
        });
        assert_eq!(result, &[1, 2, 3]);
    }
}
