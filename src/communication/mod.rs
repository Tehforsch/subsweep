use bevy::prelude::Deref;
use bevy::prelude::DerefMut;

mod collective_communicator;
mod communicated_option;
mod data_by_rank;
mod exchange_communicator;
mod from_communicator;
mod identified;
mod plugin;
mod sized_communicator;
mod sync_communicator;
mod world_communicator;

pub use collective_communicator::CollectiveCommunicator;
pub use collective_communicator::SumCommunicator;
pub use communicated_option::CommunicatedOption;
pub use data_by_rank::DataByRank;
pub use identified::Identified;
pub use plugin::BaseCommunicationPlugin;
pub use plugin::CommunicationPlugin;
pub use plugin::CommunicationType;
pub use sized_communicator::SizedCommunicator;
pub use world_communicator::WorldCommunicator;
#[cfg(feature = "local")]
mod local;

#[cfg(feature = "local")]
pub mod local_app_building;

#[cfg(feature = "local")]
pub use local_reexport::*;

#[cfg(feature = "local")]
#[path = ""]
mod local_reexport {
    use super::identified::Identified;
    pub use super::local_app_building::build_local_communication_app;
    pub use super::local_app_building::build_local_communication_app_with_custom_logic;

    pub type AllReduceCommunicator<T> = super::local::LocalCommunicator<T>;
    pub type AllGatherCommunicator<T> = super::local::LocalCommunicator<T>;
    pub type ExchangeCommunicator<T> =
        super::exchange_communicator::ExchangeCommunicator<super::local::LocalCommunicator<T>, T>;
    pub type SyncCommunicator<T> = super::sync_communicator::SyncCommunicator<
        super::local::LocalCommunicator<Identified<T>>,
        T,
    >;
    pub type Communicator<T> = super::local::LocalCommunicator<T>;
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

    pub use super::mpi_world::MpiWorld;
    pub use super::mpi_world::MPI_UNIVERSE;
}

pub type Rank = mpi::Rank;

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct WorldSize(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct WorldRank(pub Rank);

impl WorldRank {
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    pub fn main() -> Rank {
        0
    }
}
