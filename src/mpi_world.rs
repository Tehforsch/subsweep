use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Communicator;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Threading;

#[derive(Clone)]
pub struct MpiWorld {
    world: SystemCommunicator,
}

impl MpiWorld {
    pub fn initialize() -> (Universe, Self) {
        let (universe, _) = mpi::initialize_with_threading(Threading::Multiple).unwrap();
        let world = universe.world();
        (universe, Self { world })
    }
}

impl<T> crate::communication::Communicator<T> for MpiWorld
where
    T: Equivalence,
{
    fn send_vec(&mut self, rank: Rank, data: Vec<T>) {
        let num = data.len();
        let process = self.world.process_at_rank(rank);
        process.send(&num);
        for d in data.into_iter() {
            process.send(&d);
        }
    }

    fn receive_vec(&mut self, rank: Rank) -> Vec<T> {
        let process = self.world.process_at_rank(rank);
        let (num_received, _): (usize, Status) = process.receive();
        if num_received > 0 {
            let (data, _) = process.receive_vec();
            return data;
        }
        vec![]
    }

    fn rank(&self) -> i32 {
        self.world.rank()
    }

    fn size(&self) -> usize {
        self.world.size() as usize
    }
}
