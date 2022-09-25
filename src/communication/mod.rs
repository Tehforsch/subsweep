use bevy::prelude::Deref;
use bevy::prelude::DerefMut;

mod communicated_option;
mod data_by_rank;
pub mod exchange_communicator; // public because i (currently) cannot test mpi stuff from within this module, but require an externally run example for it
mod identified;
mod plugin;
mod sized_communicator;
pub mod sync_communicator; // public because i (currently) cannot test mpi stuff from within this module, but require an externally run example for it

use bevy::prelude::NonSendMut;
pub use communicated_option::CommunicatedOption;
pub use data_by_rank::DataByRank;
pub use identified::Identified;
pub use plugin::BaseCommunicationPlugin;
pub use plugin::CommunicationPlugin;
pub use plugin::CommunicationType;
pub use sized_communicator::SizedCommunicator;

pub type Communicator<'a, T> = NonSendMut<'a, communicator::Communicator<T>>;
pub type ExchangeCommunicator<'a, T> =
    NonSendMut<'a, exchange_communicator::ExchangeCommunicator<T>>;
pub type SyncCommunicator<'a, T> = NonSendMut<'a, sync_communicator::SyncCommunicator<T>>;

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
