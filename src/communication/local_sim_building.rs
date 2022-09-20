use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
use mpi::Tag;

use super::BaseCommunicationPlugin;
use crate::communication::local::LocalCommunicator;
use crate::communication::local::Payload;
use crate::communication::plugin::add_communicator;
use crate::communication::plugin::get_next_tag;
use crate::communication::CommunicationPlugin;
use crate::communication::DataByRank;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::simulation::Simulation;
use crate::simulation::TenetPlugin;

fn create_and_build_sim<F: 'static + Sync + Send + Copy + Fn(&mut Simulation)>(
    build_sim: F,
    receivers: Receivers,
    senders: Senders,
    num_threads: usize,
    rank: Rank,
) -> Simulation {
    let mut sim = Simulation::new();
    sim.add_plugin(BaseCommunicationPlugin::new(num_threads, rank));
    sim.insert_non_send_resource(receivers);
    sim.insert_non_send_resource(senders);
    build_sim(&mut sim);
    sim
}

pub fn build_local_communication_sim<F: 'static + Sync + Copy + Send + Fn(&mut Simulation)>(
    build_sim: F,
    num_threads: usize,
) {
    build_local_communication_sim_with_custom_logic(
        build_sim,
        |mut sim: Simulation| sim.run(),
        num_threads,
    );
}

pub fn build_local_communication_sim_with_custom_logic<
    F: 'static + Sync + Copy + Send + Fn(&mut Simulation),
    G: 'static + Sync + Copy + Send + Fn(Simulation),
>(
    build_sim: F,
    custom_logic: G,
    num_threads: usize,
) {
    let mut sim = create_and_build_sim(
        build_sim,
        Receivers(HashMap::new()),
        Senders(HashMap::new()),
        num_threads,
        0,
    );
    let mut handles = vec![];
    for rank in 1..num_threads {
        let receivers = Receivers({
            let all = &mut sim.unwrap_non_send_resource_mut::<Receivers>();
            let to_move = all
                .drain_filter(|comm, _| comm.owner == rank as Rank)
                .collect();
            to_move
        });
        let senders = Senders({
            let all = &mut sim.unwrap_non_send_resource_mut::<Senders>();
            let to_move = all
                .drain_filter(|comm, _| comm.owner == rank as Rank)
                .collect();
            to_move
        });
        let handle = thread::spawn(move || {
            let sim =
                create_and_build_sim(build_sim, receivers, senders, num_threads, rank as Rank);
            custom_logic(sim);
        });
        handles.push(handle);
    }
    custom_logic(sim);
    for handle in handles {
        handle.join().unwrap();
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub(super) struct Comm {
    owner: Rank,
    other: Rank,
    tag: Tag,
}

#[derive(Deref, DerefMut)]
pub(super) struct Receivers(HashMap<Comm, Receiver<Payload>>);

#[derive(Deref, DerefMut)]
struct Senders(HashMap<Comm, Sender<Payload>>);

impl<T> TenetPlugin for CommunicationPlugin<T>
where
    T: Equivalence + Sync + Send + 'static,
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build_everywhere(&self, sim: &mut Simulation) {
        let tag = get_next_tag(sim);
        let rank = **sim.unwrap_resource::<WorldRank>();
        let world_size = **sim.unwrap_resource::<WorldSize>();
        if rank == 0 {
            let (senders, receivers) = get_senders_and_receivers(world_size, tag);
            sim.unwrap_non_send_resource_mut::<Senders>()
                .extend(senders.into_iter());
            sim.unwrap_non_send_resource_mut::<Receivers>()
                .extend(receivers.into_iter());
        }
        let mut commun = LocalCommunicator::<T>::new(
            DataByRank::empty(),
            DataByRank::empty(),
            tag,
            world_size,
            rank,
        );
        let mut senders = sim.unwrap_non_send_resource_mut::<Senders>();
        add_senders_to_communicator(&mut commun, &mut senders);
        let mut receivers = sim.unwrap_non_send_resource_mut::<Receivers>();
        add_receivers_to_communicator(&mut commun, &mut receivers);
        add_communicator(self.type_, sim, commun);
    }

    fn allow_adding_twice(&self) -> bool {
        true
    }
}

pub(super) fn add_senders_to_communicator<T>(
    communicator: &mut LocalCommunicator<T>,
    senders: &mut HashMap<Comm, Sender<Payload>>,
) {
    for r in communicator.other_ranks() {
        let sender = senders
            .remove(&Comm {
                owner: communicator.rank(),
                other: r,
                tag: communicator.tag(),
            })
            .unwrap();
        communicator.senders.insert(r, sender);
    }
}

pub(super) fn add_receivers_to_communicator<T>(
    communicator: &mut LocalCommunicator<T>,
    receivers: &mut HashMap<Comm, Receiver<Payload>>,
) {
    for r in communicator.other_ranks() {
        let receiver = receivers
            .remove(&Comm {
                owner: communicator.rank(),
                other: r,
                tag: communicator.tag(),
            })
            .unwrap();
        communicator.receivers.insert(r, receiver);
    }
}

pub(super) fn get_senders_and_receivers(
    num_threads: usize,
    tag: Tag,
) -> (
    HashMap<Comm, Sender<Payload>>,
    HashMap<Comm, Receiver<Payload>>,
) {
    let mut senders = HashMap::new();
    let mut receivers = HashMap::new();
    for rank1 in 0i32..num_threads as i32 {
        for rank2 in 0i32..num_threads as i32 {
            if rank1 == rank2 {
                continue;
            }
            let (sender1, receiver1) = channel();
            let (sender2, receiver2) = channel();
            receivers.insert(
                Comm {
                    owner: rank1,
                    other: rank2,
                    tag,
                },
                receiver1,
            );
            receivers.insert(
                Comm {
                    owner: rank2,
                    other: rank1,
                    tag,
                },
                receiver2,
            );
            senders.insert(
                Comm {
                    owner: rank2,
                    other: rank1,
                    tag,
                },
                sender1,
            );
            senders.insert(
                Comm {
                    owner: rank1,
                    other: rank2,
                    tag,
                },
                sender2,
            );
        }
    }
    (senders, receivers)
}
