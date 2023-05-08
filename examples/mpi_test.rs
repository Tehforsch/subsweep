// This is effectively an MPI test written as an example. This is
// unfortunate but "necessary" since support for custom test runners
// is very light at the moment.  I tried obtaining the executable
// built by cargo test and then running it with mpirun (filtering only
// for the tests that should be run with mpirun), but this doesn't
// work because of how cargo test produces a multithreaded binary -
// thread/rank 0 gets stuck at distributing work and doesn't enter the
// program.  Passing --num-threads 1 and --jobs 1 does not help

use std::thread;
use std::time::Duration;

use mpi::traits::Communicator;
use mpi::Tag;
use raxiom::communication::exchange_communicator::ExchangeCommunicator;
use raxiom::communication::DataByRank;
use raxiom::communication::MpiWorld;
use raxiom::communication::SizedCommunicator;
use raxiom::communication::MPI_UNIVERSE;
use raxiom::prelude::ParticleId;
use raxiom::sweep::DirectionIndex;
use raxiom::sweep::FluxData;
use raxiom::sweep::SweepCommunicator;
use raxiom::units::PhotonRate;

pub fn main() {
    let fns: &[(&str, fn())] = &[
        ("exchange_all", exchange_all),
        ("send_receive", send_receive),
        ("sweep_communicator", sweep_communicator),
    ];
    for (name, f) in fns {
        f();
        if MPI_UNIVERSE.world().rank() == 0 {
            println!("{} ... ok", name);
        }
    }
    MPI_UNIVERSE.drop();
}

fn send_receive() {
    let mut world = MpiWorld::<i32>::new(Tag::default());
    let rank = world.rank();
    let x0: Vec<i32> = vec![1, 2, 3];
    let x1: Vec<i32> = vec![3, 2, 1];
    if rank == 0 {
        world.blocking_send_vec(1, &x0);
        assert_eq!(MpiWorld::<i32>::receive_vec(&mut world, 1), x1);
    } else if rank == 1 {
        assert_eq!(MpiWorld::<i32>::receive_vec(&mut world, 0), x0);
        world.blocking_send_vec(0, &x1);
    }
}

fn exchange_all() {
    let world = MpiWorld::<i32>::new(Tag::default());
    let rank = world.rank();
    let mut exchange_comm = ExchangeCommunicator::from(world);
    for _ in 0..100 {
        let x0: Vec<i32> = (0..100).collect();
        let x1: Vec<i32> = (0..100).rev().collect();
        if rank == 0 {
            let data = DataByRank::from_iter([(1, x0)].into_iter());
            let res = exchange_comm.exchange_all(data);
            assert_eq!(res[1], x1);
        } else if rank == 1 {
            let data = DataByRank::from_iter([(0, x1)].into_iter());
            let res = exchange_comm.exchange_all(data);
            assert_eq!(res[0], x0);
        }
    }
}

fn sweep_communicator() {
    let mut world = MpiWorld::<FluxData>::new(Tag::default());
    let rank = world.rank();
    let mut comm = SweepCommunicator::new(&mut world);
    let size = 10000;
    let num_iterations = 100;
    let make_data = |to_rank| {
        let f = FluxData {
            dir: DirectionIndex(0),
            flux: PhotonRate::zero(),
            id: ParticleId(0),
        };
        // Make this large so that it will require buffered communication
        DataByRank::from_iter([(to_rank, (0..size).map(|_| f.clone()).collect())])
    };
    if rank == 0 {
        let mut num_received = 0;
        while num_received < size * num_iterations {
            if let Some(recv) = comm.try_recv(1) {
                num_received += recv.len();
                assert_eq!(recv[0].dir.0, 0);
                assert_eq!(recv[0].id.0, 0);
                thread::sleep(Duration::from_millis(10));
            }
        }
    } else if rank == 1 {
        let mut data: DataByRank<Vec<_>> = make_data(0);
        for _ in 0..num_iterations {
            data[0].extend(make_data(0).remove(&0).unwrap().into_iter());
            comm.try_send_all(&mut data);
            thread::sleep(Duration::from_millis(10));
        }
        while data.size() > 0 {
            comm.try_send_all(&mut data);
        }
    }
}
