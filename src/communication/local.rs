use std::iter::Sum;
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use mpi::request::Scope;
use mpi::request::WaitGuard;
use mpi::Count;
use mpi::Tag;

use super::collective_communicator::SumCommunicator;
use super::sized_communicator::SizedCommunicator;
use super::world_communicator::WorldCommunicator;
use super::CollectiveCommunicator;
use super::DataByRank;
use super::Identified;
use super::Rank;

pub(super) struct Payload {
    bytes: Vec<u8>,
}

pub struct LocalCommunicator<T> {
    pub(super) senders: DataByRank<Sender<Payload>>,
    pub(super) receivers: DataByRank<Receiver<Payload>>,
    rank: Rank,
    size: usize,
    _marker: PhantomData<T>,
    tag: Tag,
}

impl<T> LocalCommunicator<T> {
    pub(super) fn new(
        receivers: DataByRank<Receiver<Payload>>,
        senders: DataByRank<Sender<Payload>>,
        tag: Tag,
        size: usize,
        rank: Rank,
    ) -> Self {
        Self {
            senders,
            receivers,
            rank,
            size,
            tag: tag,
            _marker: PhantomData::default(),
        }
    }

    pub(super) fn tag(&self) -> Tag {
        self.tag
    }
}

impl<T: Sync + Send> WorldCommunicator<T> for LocalCommunicator<T> {
    fn receive_vec(&mut self, rank: Rank) -> Vec<T> {
        let bytes = &self.receivers[rank].recv().unwrap().bytes;
        let size = mem::size_of::<T>();
        debug_assert_eq!(bytes.len().rem_euclid(size), 0);
        bytes
            .chunks_exact(size)
            .map(|chunk| unsafe { ptr::read(chunk.as_ptr().cast()) })
            .collect()
    }

    fn blocking_send_vec(&mut self, rank: Rank, data: &[T]) {
        let bytes = unsafe {
            slice::from_raw_parts(
                (data as *const [T]) as *const u8,
                data.len() * mem::size_of::<T>(),
            )
        };
        let payload = Payload {
            bytes: bytes.to_vec(),
        };
        self.senders[rank].send(payload).unwrap();
    }

    fn immediate_send_vec<'a, Sc: Scope<'a>>(
        &mut self,
        _scope: Sc,
        rank: Rank,
        data: &'a [T],
    ) -> Option<WaitGuard<'a, [T], Sc>> {
        // Local communication does not block anyways
        self.blocking_send_vec(rank, data);
        None
    }
}

impl<T> SizedCommunicator for LocalCommunicator<T> {
    fn rank(&self) -> Rank {
        self.rank
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl<T: Clone + Sync + Send> CollectiveCommunicator<T> for LocalCommunicator<T> {
    fn all_gather(&mut self, data: &T) -> Vec<T> {
        self.all_gather_vec(&[data.clone()])
            .drain_all()
            .flat_map(|(_, data)| data)
            .collect()
    }

    fn all_gather_vec(&mut self, data: &[T]) -> DataByRank<Vec<T>> {
        for rank in self.other_ranks() {
            self.blocking_send_vec(rank, data);
        }
        let mut result = DataByRank::empty();
        for rank in self.all_ranks() {
            if rank == self.rank {
                result.insert(rank, data.to_vec());
            } else {
                let received = self.receive_vec(rank);
                result.insert(rank, received);
            }
        }
        result
    }

    fn all_gather_varcount(&mut self, data: &[T], _counts: &[Count]) -> Vec<T> {
        for rank in self.other_ranks() {
            self.blocking_send_vec(rank, data);
        }
        let mut result = vec![];
        for rank in self.all_ranks() {
            if rank == self.rank {
                result.extend(data.to_vec());
            } else {
                let received = self.receive_vec(rank);
                result.extend(received.into_iter());
            }
        }
        result
    }
}

impl<T: Sum + Clone + Sync + Send> SumCommunicator<T> for LocalCommunicator<T> {
    fn collective_sum(&mut self, send: &T) -> T {
        // We don't care about efficiency in the local communicator
        let result = self.all_gather(send);
        result.into_iter().sum()
    }
}

impl<T> From<LocalCommunicator<T>> for LocalCommunicator<Identified<T>> {
    fn from(other: LocalCommunicator<T>) -> Self {
        Self {
            senders: other.senders,
            receivers: other.receivers,
            rank: other.rank,
            size: other.size,
            _marker: PhantomData::default(),
            tag: other.tag,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::communication::plugin::INITIAL_TAG;
    use crate::communication::sync_communicator::tests::get_communicators;
    use crate::communication::CollectiveCommunicator;
    use crate::communication::WorldCommunicator;

    #[derive(Clone, Debug, PartialEq)]
    struct ComplexStruct {
        a: f64,
        b: u8,
    }

    #[test]
    fn local_communicator_struct() {
        let mut comms = get_communicators(2, INITIAL_TAG);
        let mut comm0 = comms.remove(&0).unwrap();
        let mut comm1 = comms.remove(&1).unwrap();
        let x = ComplexStruct { a: 1.5, b: 13 };
        let xs = (0..100)
            .map(|num| ComplexStruct {
                a: num as f64 * 0.1,
                b: num,
            })
            .collect::<Vec<_>>();
        comm0.blocking_send_vec(1, &[x.clone()]);
        assert_eq!(comm1.receive_vec(0), vec![x]);
        comm0.blocking_send_vec(1, &xs.clone());
        assert_eq!(comm1.receive_vec(0), xs.clone());
    }

    #[test]
    fn local_communicator_i32() {
        let mut comms = get_communicators(2, INITIAL_TAG);
        let mut comm0 = comms.remove(&0).unwrap();
        let mut comm1 = comms.remove(&1).unwrap();
        let xs: Vec<i32> = vec![42, 0x01020304, 3];
        comm0.blocking_send_vec(1, &xs.clone());
        assert_eq!(comm1.receive_vec(0), xs);
    }

    #[test]
    fn local_communicator_mixed_types() {
        let mut comms = get_communicators(2, INITIAL_TAG);
        let mut comm_a0 = comms.remove(&0).unwrap();
        let mut comm_a1 = comms.remove(&1).unwrap();
        let mut comms = get_communicators(2, INITIAL_TAG + 1);
        let mut comm_b0 = comms.remove(&0).unwrap();
        let mut comm_b1 = comms.remove(&1).unwrap();
        let xs_a: Vec<i32> = vec![1, 2, 3];
        let xs_b: Vec<f64> = vec![1.0, 2.0, 3.0];
        comm_a0.blocking_send_vec(1, &xs_a.clone());
        comm_b0.blocking_send_vec(1, &xs_b.clone());
        assert_eq!(comm_a1.receive_vec(0), xs_a);
        assert_eq!(comm_b1.receive_vec(0), xs_b);
    }

    #[test]
    fn local_communicator_allgather_vec() {
        let mut comms = get_communicators(2, INITIAL_TAG);
        let mut comm0 = comms.remove(&0).unwrap();
        let mut comm1 = comms.remove(&1).unwrap();
        let xs_0: Vec<f64> = vec![1.0, 3.0, 5.0];
        let xs_1: Vec<f64> = vec![1.0, 2.0, 3.0];
        let xs_0_cloned: Vec<f64> = vec![1.0, 3.0, 5.0];
        let xs_1_cloned: Vec<f64> = vec![1.0, 2.0, 3.0];
        let h = thread::spawn(move || {
            let result = comm0.all_gather_vec(&xs_0.clone());
            assert_eq!(result[0], xs_0);
            assert_eq!(result[1], xs_1);
        });
        thread::spawn(move || {
            let result = comm1.all_gather_vec(&xs_1_cloned.clone());
            assert_eq!(result[0], xs_0_cloned);
            assert_eq!(result[1], xs_1_cloned);
        })
        .join()
        .unwrap();
        h.join().unwrap();
    }
}
