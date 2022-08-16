use super::DataByRank;
use super::Rank;

pub trait Communicator<T> {
    fn send_vec(&mut self, rank: Rank, data: Vec<T>);
    fn receive_vec(&mut self, rank: Rank) -> Vec<T>;
    fn size(&self) -> usize;
    fn rank(&self) -> Rank;

    fn other_ranks(&self) -> Vec<Rank> {
        (0i32..self.size() as i32)
            .filter(|rank| *rank != self.rank())
            .collect()
    }

    fn initialize_data_by_rank(&self) -> DataByRank<Vec<T>> {
        let size = self.size();
        let rank = self.rank();
        DataByRank::new(size, rank)
    }
}
