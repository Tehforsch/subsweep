use bevy::prelude::Deref;
use bevy::prelude::DerefMut;

mod communicated_option;
mod data_by_rank;
mod exchange_communicator;
mod identified;
mod plugin;
mod sized_communicator;
mod sync_communicator;

pub use communicated_option::CommunicatedOption;
pub use data_by_rank::DataByRank;
pub use identified::Identified;
pub use plugin::BaseCommunicationPlugin;
pub use plugin::CommunicationPlugin;
pub use plugin::CommunicationType;
pub use sized_communicator::SizedCommunicator;

// Will be used eventually, so allow dead code for now
#[allow(dead_code)]
pub type AllReduceCommunicator<T> = communicator::Communicator<T>;
pub type AllGatherCommunicator<T> = communicator::Communicator<T>;
pub type ExchangeCommunicator<T> = exchange_communicator::ExchangeCommunicator<T>;
pub type SyncCommunicator<T> = sync_communicator::SyncCommunicator<T>;

#[cfg(feature = "mpi")]
mod verify_tag_type_mapping;

#[cfg(not(feature = "mpi"))]
mod local;

#[cfg(not(feature = "mpi"))]
pub use local_reexport::*;

#[cfg(not(feature = "mpi"))]
#[path = ""]
mod local_reexport {
    pub use super::local_sim_building::build_local_communication_sim;
    pub use super::local_sim_building::build_local_communication_sim_with_custom_logic;

    pub mod local_sim_building;

    pub(super) mod communicator {
        pub type Communicator<T> = super::super::local::LocalCommunicator<T>;
    }
}

#[cfg(feature = "mpi")]
mod mpi_world;

#[cfg(feature = "mpi")]
pub use mpi_reexport::*;

#[cfg(feature = "mpi")]
#[path = ""]
mod mpi_reexport {
    pub use super::mpi_world::MpiWorld;
    pub use super::mpi_world::MPI_UNIVERSE;

    pub(super) mod communicator {
        pub type Communicator<T> = super::super::mpi_world::MpiWorld<T>;
    }
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
