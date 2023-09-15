use derive_more::Deref;
use derive_more::DerefMut;

mod communicated_option;
mod data_by_rank;
pub mod exchange_communicator; // public because i (currently) cannot test mpi stuff from within this module, but require an externally run example for it
mod identified;
mod plugin;
mod sized_communicator;
pub mod sync_communicator; // public because i (currently) cannot test mpi stuff from within this module, but require an externally run example for it

use bevy_ecs::prelude::Resource;
pub use communicated_option::CommunicatedOption;
pub use data_by_rank::DataByRank;
pub use exchange_communicator::ExchangeCommunicator;
pub use identified::EntityKey;
pub use identified::Identified;
pub use plugin::BaseCommunicationPlugin;
pub use sized_communicator::SizedCommunicator;

mod mpi_world;

pub use self::mpi_world::MpiWorld;
pub use self::mpi_world::MPI_UNIVERSE;

pub mod communicator {
    pub type Communicator<T> = super::mpi_world::MpiWorld<T>;
}

pub type Rank = mpi::Rank;

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut, Resource)]
pub struct WorldSize(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut, Resource, Hash)]
pub struct WorldRank(pub Rank);

impl WorldRank {
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    pub fn main() -> Rank {
        0
    }
}
