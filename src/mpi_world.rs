use bevy::prelude::App;
use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Communicator;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Threading;

use crate::communication::BufferedCommunicator;
use crate::communication::DataByRank;

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
}

pub struct MpiCommunicator<'a, T> {
    world: &'a MpiWorld,
    data: DataByRank<Vec<T>>,
}

impl<'a, T> MpiCommunicator<'a, T> {
    pub fn new(world: &'a MpiWorld) -> Self {
        Self {
            world,
            data: Self::get_data_by_rank(&world),
        }
    }

    fn get_data_by_rank(world: &MpiWorld) -> DataByRank<Vec<T>> {
        let size = world.size() as usize;
        let rank = world.rank();
        DataByRank::new(size, rank)
    }
}

impl<'a, T> BufferedCommunicator<T> for MpiCommunicator<'a, T>
where
    T: Equivalence,
{
    fn send(&mut self, rank: i32, data: T) {
        self.data.push(rank, data);
    }

    fn receive_vec(self) -> DataByRank<Vec<T>> {
        for rank in self.world.other_ranks() {
            let num = self.data.get(&rank).map(|data| data.len()).unwrap_or(0);
            self.world.send(rank, num);
        }
        for (rank, data) in self.data.into_iter() {
            for d in data.into_iter() {
                self.world.send(rank, d);
            }
        }
        let mut received_data = Self::get_data_by_rank(&self.world);
        for rank in self.world.other_ranks() {
            let (num_incoming, _): (usize, Status) =
                self.world.world().process_at_rank(rank).receive();
            if num_incoming > 0 {
                let (moved_to_own_domain, _): (Vec<T>, Status) =
                    self.world.world().process_at_rank(rank).receive_vec();
                received_data.insert(rank, moved_to_own_domain);
            }
        }
        received_data
    }
}
