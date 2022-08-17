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
}
