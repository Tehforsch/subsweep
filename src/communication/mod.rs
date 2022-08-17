mod data_by_rank;
mod exchange_communicator;
mod world_communicator;

#[cfg(not(feature = "local"))]
mod mpi_world;

#[cfg(feature = "local")]
mod local;

pub use data_by_rank::DataByRank;
pub use world_communicator::WorldCommunicator;

#[cfg(feature = "local")]
pub use self::local::get_local_communicators;

pub type Rank = mpi::Rank;

#[cfg(feature = "local")]
pub type ExchangeCommunicator<T> =
    exchange_communicator::ExchangeCommunicator<self::local::LocalCommunicator<T>, T>;

#[cfg(not(feature = "local"))]
pub type ExchangeCommunicator<T> =
    exchange_communicator::ExchangeCommunicator<self::mpi_world::MpiWorld<T>, T>;

#[cfg(feature = "local")]
pub type Communicator<T> = self::local::LocalCommunicator<T>;
#[cfg(not(feature = "local"))]
pub type Communicator<T> = self::mpi_world::MpiWorld<T>;
