mod communicator;
mod data_by_rank;
mod exchange_communicator;

pub type Rank = mpi::Rank;

pub use communicator::Communicator;
pub use data_by_rank::DataByRank;
pub use exchange_communicator::ExchangeCommunicator;
