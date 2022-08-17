use super::DataByRank;
use super::Rank;

pub trait SizedCommunicator {
    fn size(&self) -> usize;
    fn rank(&self) -> Rank;

    fn other_ranks(&self) -> Vec<Rank> {
        (0i32..self.size() as i32)
            .filter(|rank| *rank != self.rank())
            .collect()
    }

    fn initialize_data_by_rank<T>(&self) -> DataByRank<Vec<T>> {
        let size = self.size();
        let rank = self.rank();
        DataByRank::new(size, rank)
    }
}
