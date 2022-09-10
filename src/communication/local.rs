use std::iter::Sum;
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

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
    marker_: PhantomData<T>,
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
            marker_: PhantomData::default(),
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

    fn send_vec(&mut self, rank: Rank, data: Vec<T>) {
        let bytes = unsafe {
            slice::from_raw_parts(
                (data.as_slice() as *const [T]) as *const u8,
                data.len() * mem::size_of::<T>(),
            )
        };
        let payload = Payload {
            bytes: bytes.to_vec(),
        };
        self.senders[rank].send(payload).unwrap();
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
        for rank in self.other_ranks() {
            self.send_vec(rank, vec![data.clone()]);
        }
        let mut result = vec![];
        for rank in self.all_ranks() {
            if rank == self.rank {
                result.push(data.clone());
            } else {
                let received = self.receive_vec(rank);
                assert_eq!(received.len(), 1);
                result.extend(received.into_iter());
            }
        }
        result
    }

    fn all_gather_varcount(&mut self, data: &[T], _counts: &[Count]) -> Vec<T> {
        for rank in self.other_ranks() {
            self.send_vec(rank, data.to_vec());
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
            marker_: PhantomData::default(),
            tag: other.tag,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::communication::plugin::INITIAL_TAG;
    use crate::communication::sync_communicator::tests::get_communicators;
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
        comm0.send_vec(1, vec![x.clone()]);
        assert_eq!(comm1.receive_vec(0), vec![x]);
        comm0.send_vec(1, xs.clone());
        assert_eq!(comm1.receive_vec(0), xs.clone());
    }

    #[test]
    fn local_communicator_i32() {
        let mut comms = get_communicators(2, INITIAL_TAG);
        let mut comm0 = comms.remove(&0).unwrap();
        let mut comm1 = comms.remove(&1).unwrap();
        let xs: Vec<i32> = vec![42, 0x01020304, 3];
        comm0.send_vec(1, xs.clone());
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
        let xs_b: Vec<f32> = vec![1.0, 2.0, 3.0];
        comm_a0.send_vec(1, xs_a.clone());
        comm_b0.send_vec(1, xs_b.clone());
        assert_eq!(comm_a1.receive_vec(0), xs_a);
        assert_eq!(comm_b1.receive_vec(0), xs_b);
    }
}
