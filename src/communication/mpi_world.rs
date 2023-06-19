use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter::Sum;
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
use mpi::request::Request;
use mpi::request::Scope;
use mpi::request::WaitGuard;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Communicator;
use mpi::traits::CommunicatorCollectives;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::MatchedReceiveVec;
use mpi::traits::Source;
use mpi::Count;
use mpi::Tag;
use mpi::Threading;

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
        let (mut universe, threading_initialized) =
            mpi::initialize_with_threading(threading).unwrap();
        universe.set_buffer_size(1024 * 16);
        assert_eq!(
            threading, threading_initialized,
            "Could not initialize MPI with Multithreading"
        );
        StaticUniverse(Arc::new(Mutex::new(Some(universe))))
    };
}

fn get_hash_for_type<T: 'static>() -> u64 {
    let id = TypeId::of::<T>();
    let mut s = DefaultHasher::new();
    id.hash(&mut s);
    s.finish()
}

fn get_tag_for_type<T: 'static>() -> Tag {
    let hash: u64 = get_hash_for_type::<T>();
    // Silently truncate 3/4 of the bits in the integer and then take the absolute value to make sure we have no negative values.
    // This is hacky but feels better than not having tags at all. Collision chance should still be negligible.
    (hash as i16).abs() as i32
}

#[derive(Clone)]
pub struct MpiWorld<T> {
    world: SystemCommunicator,
    _marker: PhantomData<T>,
    tag: Tag,
}

impl<T: 'static> MpiWorld<T> {
    pub fn new() -> Self {
        let world = MPI_UNIVERSE.world();
        let tag = get_tag_for_type::<T>();
        Self {
            world,
            _marker: PhantomData::default(),
            tag,
        }
    }

    pub fn new_custom_tag(tag: Tag) -> Self {
        let world = MPI_UNIVERSE.world();
        Self {
            world,
            tag,
            _marker: PhantomData::default(),
        }
    }
}

impl<T> MpiWorld<T>
where
    T: Equivalence,
{
    /// Should be called before any collective operation.  Checks that
    /// the tag being communicated is the same across all ranks. This
    /// is an additional check to prevent any kind of mixing of the
    /// type involved in collective operations. This is done
    /// explicitly here since collective MPI operations do not support
    /// tags.
    fn verify_tag(&mut self) {
        let tag = self.tag;
        debug_assert!(self.all_ranks_have_same_value(&tag), "Initializing allgather operation but different ranks have different tags. Tag on rank {}: {}!", self.world.rank(), self.tag)
    }

    fn unchecked_convert<S>(&self) -> MpiWorld<S> {
        MpiWorld::<S> {
            world: self.world,
            _marker: PhantomData,
            tag: self.tag,
        }
    }
}

impl<S> MpiWorld<S>
where
    S: Equivalence,
{
    pub fn receive_vec(&mut self, rank: Rank) -> Vec<S> {
        let process = self.world.process_at_rank(rank);
        let result = process.matched_probe_with_tag(self.tag);
        let (data, _) = result.matched_receive_vec();
        data
    }

    pub fn try_receive_vec(&mut self, rank: Rank) -> Option<Vec<S>> {
        let process = self.world.process_at_rank(rank);
        let result = process.immediate_matched_probe_with_tag(self.tag);
        result.map(|result| {
            let (data, _) = result.matched_receive_vec();
            data
        })
    }

    pub fn blocking_send_vec(&mut self, rank: Rank, data: &[S]) {
        let process = self.world.process_at_rank(rank);
        process.send_with_tag(data, self.tag);
    }

    #[must_use]
    pub fn immediate_send_vec<'a, Sc: Scope<'a>>(
        &mut self,
        scope: Sc,
        rank: Rank,
        data: &'a [S],
    ) -> Option<Request<'a, [S], Sc>> {
        let process = self.world.process_at_rank(rank);
        Some(process.immediate_send_with_tag(scope, data, self.tag))
    }

    #[must_use]
    pub fn immediate_send_vec_wait_guard<'a, Sc: Scope<'a>>(
        &mut self,
        scope: Sc,
        rank: Rank,
        data: &'a [S],
    ) -> Option<WaitGuard<'a, [S], Sc>> {
        self.immediate_send_vec(scope, rank, data)
            .map(WaitGuard::from)
    }
}

impl<S> MpiWorld<S>
where
    S: Equivalence + Clone,
{
    // Temporary replacement for a proper AllReduce call
    pub fn all_gather_sum<T>(&mut self, send: &S) -> T
    where
        T: Sum<T> + From<S>,
    {
        self.verify_tag();
        unchecked_all_gather(self.world, send)
            .into_iter()
            .map(|s| T::from(s))
            .sum()
    }

    // Temporary replacement for a proper AllReduce call
    pub fn all_gather_min<T>(&mut self, send: &S) -> Option<T>
    where
        T: PartialOrd<T> + From<S>,
    {
        self.verify_tag();
        unchecked_all_gather(self.world, send)
            .into_iter()
            .map(|s| T::from(s))
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
    }

    // Temporary replacement for a proper AllReduce call
    pub fn all_gather_max<T>(&mut self, send: &S) -> Option<T>
    where
        T: PartialOrd<T> + From<S>,
    {
        self.verify_tag();
        unchecked_all_gather(self.world, send)
            .into_iter()
            .map(|s| T::from(s))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn all_gather(&mut self, send: &S) -> Vec<S> {
        self.verify_tag();
        unchecked_all_gather(self.world, send)
    }

    pub fn all_reduce_sum(&mut self, send: &u64) -> u64 {
        let mut sum = 0u64;
        self.world
            .all_reduce_into(send, &mut sum, SystemOperation::sum());
        sum
    }

    pub fn all_gather_vec(&mut self, send: &[S]) -> DataByRank<Vec<S>> {
        self.verify_tag();
        let world_size = self.world.size() as usize;
        let num_elements = send.len();
        let mut result_buffer = unsafe { get_buffer::<S>(world_size * num_elements) };
        self.world.all_gather_into(send, &mut result_buffer[..]);
        let mut data = DataByRank::empty();
        for i in 0..world_size {
            data.insert(i as Rank, result_buffer.drain(0..num_elements).collect())
        }
        data
    }

    fn all_gather_varcount_with_counts(&mut self, send: &[S], counts: &[Count]) -> Vec<S> {
        self.verify_tag();
        let mut result_buffer: Vec<S> =
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

    pub fn all_gather_varcount(&mut self, send: &[S]) -> Vec<S> {
        let mut counter: MpiWorld<usize> = self.unchecked_convert();
        let counts = counter.all_gather(&send.len());
        let counts: Vec<_> = counts.into_iter().map(|x| x as Count).collect();
        self.all_gather_varcount_with_counts(send, &counts)
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

impl<S> MpiWorld<S> {
    pub fn all_ranks_have_same_value<T: Equivalence + PartialEq>(&mut self, value: &T) -> bool {
        let values = unchecked_all_gather(self.world, value);
        for other_value in values {
            if *value != other_value {
                return false;
            }
        }
        true
    }
}

fn unchecked_all_gather<T: Equivalence>(world: SystemCommunicator, send: &T) -> Vec<T> {
    let mut result_buffer = unsafe { get_buffer(world.size() as usize) };
    world.all_gather_into(send, &mut result_buffer[..]);
    result_buffer
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

    use super::MpiWorld;

    #[test]
    fn immediate_send_receive() {
        let mut world = MpiWorld::<i32>::new();
        let x: [i32; 3] = [1, 2, 3];
        let result: Vec<i32> = scope(|scope| {
            let _guard = world.immediate_send_vec_wait_guard(scope, 0, &x);
            world.receive_vec(0)
        });
        assert_eq!(result, &[1, 2, 3]);
    }
}
