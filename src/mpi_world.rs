use bevy::prelude::App;
use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Communicator;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Threading;

pub fn initialize_mpi_and_add_world_resource(app: &mut App) -> Rank {
    let mpi_world = MpiWorld::new();
    let rank = mpi_world.rank();
    app.insert_non_send_resource(mpi_world);
    rank
}

pub struct MpiWorld {
    universe: Universe,
}

impl MpiWorld {
    pub fn new() -> Self {
        let (universe, _) = mpi::initialize_with_threading(Threading::Multiple).unwrap();
        Self { universe }
    }

    pub fn rank(&self) -> i32 {
        self.world().rank()
    }

    pub fn size(&self) -> i32 {
        self.world().size()
    }

    pub fn world(&self) -> SystemCommunicator {
        self.universe.world()
    }

    pub fn send<T: Equivalence>(&self, rank: Rank, data: T) {
        self.world().process_at_rank(rank).send(&data)
    }

    pub fn other_ranks(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.size()).filter(|rank| *rank != self.rank())
    }

    pub fn receive_any<Message: Equivalence>(&self) -> (Message, Status) {
        self.world().any_process().receive()
    }
}
