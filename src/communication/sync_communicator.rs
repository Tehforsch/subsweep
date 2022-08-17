use super::{exchange_communicator::ExchangeCommunicator, WorldCommunicator};

pub struct SyncCommunicator<C, T> where C: WorldCommunicator<T> {
    exchange_communicator: ExchangeCommunicator<C, T>,
}

impl<C, T> SyncCommunicator<C, T> where C: WorldCommunicator<T> {
    pub fn receive_vec(&mut self) {
        let data = self.exchange_communicator.receive_vec();
    }
}
