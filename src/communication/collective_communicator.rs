use mpi::Count;

use super::DataByRank;

pub trait CollectiveCommunicator<T> {
    fn all_gather(&mut self, send: &T) -> Vec<T>;
    fn all_gather_vec(&mut self, send: &[T]) -> DataByRank<Vec<T>>;
    fn all_gather_varcount(&mut self, send: &[T], counts: &[Count]) -> Vec<T>;
}

pub trait SumCommunicator<T> {
    fn collective_sum(&mut self, send: &T) -> T;
}
