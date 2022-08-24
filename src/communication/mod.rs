mod collective_communicator;
mod data_by_rank;
mod exchange_communicator;
mod identified;
mod sized_communicator;
mod sync_communicator;
mod world_communicator;

pub use collective_communicator::CollectiveCommunicator;
pub use collective_communicator::Operation;
pub use data_by_rank::DataByRank;
pub use identified::Identified;
pub use sized_communicator::SizedCommunicator;
pub use world_communicator::WorldCommunicator;

#[cfg(feature = "local")]
mod local;

#[cfg(feature = "local")]
pub use local_reexport::*;

#[cfg(feature = "local")]
#[path = ""]
mod local_reexport {
    use super::identified::Identified;

    pub type ExchangeCommunicator<T> =
        super::exchange_communicator::ExchangeCommunicator<super::local::LocalCommunicator<T>, T>;
    pub type SyncCommunicator<T> = super::sync_communicator::SyncCommunicator<
        super::local::LocalCommunicator<Identified<T>>,
        T,
    >;
    pub type Communicator<T> = super::local::LocalCommunicator<T>;

    pub use super::local::get_local_communicators;
}

#[cfg(not(feature = "local"))]
mod mpi_world;

#[cfg(not(feature = "local"))]
pub use mpi_reexport::*;

#[cfg(not(feature = "local"))]
#[path = ""]
mod mpi_reexport {
    use super::identified::Identified;
    pub type AllReduceCommunicator<T> = super::mpi_world::MpiWorld<T>;
    pub type AllGatherCommunicator<T> = super::mpi_world::MpiWorld<T>;
    pub type ExchangeCommunicator<T> =
        super::exchange_communicator::ExchangeCommunicator<super::mpi_world::MpiWorld<T>, T>;
    pub type SyncCommunicator<T> =
        super::sync_communicator::SyncCommunicator<super::mpi_world::MpiWorld<Identified<T>>, T>;

    pub type Communicator<T> = super::mpi_world::MpiWorld<T>;

    pub use super::mpi_world::MPI_UNIVERSE;
}

pub type Rank = mpi::Rank;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NumRanks(pub usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WorldRank(pub Rank);

impl WorldRank {
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    pub fn main() -> Rank {
        0
    }
}
