use super::world_communicator::WorldCommunicator;
use super::DataByRank;
use super::Rank;

pub struct ExchangeCommunicator<C, T>
where
    C: WorldCommunicator<T>,
{
    communicator: C,
    data: DataByRank<Vec<T>>,
}

impl<C, T> Clone for ExchangeCommunicator<C, T>
where
    C: Clone + WorldCommunicator<T>,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            communicator: self.communicator.clone(),
            data: self.data.clone(),
        }
    }
}

impl<C, T> ExchangeCommunicator<C, T>
where
    C: WorldCommunicator<T>,
{
    pub fn new(communicator: C) -> Self {
        let data = communicator.initialize_data_by_rank();
        Self {
            communicator,
            data: data,
        }
    }

    pub fn send(&mut self, rank: i32, data: T) {
        self.data.push(rank, data);
    }

    pub fn receive_vec(&mut self) -> DataByRank<Vec<T>> {
        for (rank, data) in self.data.drain_all() {
            self.communicator.send_vec(rank, data);
        }
        let mut received_data = self.communicator.initialize_data_by_rank();
        for rank in self.communicator.other_ranks() {
            let moved_to_own_domain = self.communicator.receive_vec(rank);
            received_data.insert(rank, moved_to_own_domain);
        }
        received_data
    }

    pub fn rank(&self) -> Rank {
        self.communicator.rank()
    }
}

#[cfg(test)]
#[cfg(feature = "local")]
mod tests {
    #[test]
    fn exchange_communicator() {}
}
