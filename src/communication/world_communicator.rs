use super::Rank;

pub trait WorldCommunicator<T> {
    fn blocking_send_vec(&mut self, rank: Rank, data: Vec<T>);
    fn receive_vec(&mut self, rank: Rank) -> Vec<T>;
}
