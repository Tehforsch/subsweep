use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use bevy::prelude::App;
use bevy::prelude::Plugin;
use mpi::Tag;

use super::*;
use crate::command_line_options::CommandLineOptions;
use crate::communication::from_communicator::FromCommunicator;
use crate::communication::local::LocalCommunicator;
use crate::communication::local::Payload;
use crate::communication::plugin::CurrentTag;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::DataByRank;
use crate::communication::NumRanks;
use crate::communication::Rank;
use crate::communication::WorldRank;

fn create_and_build_app<
    F: 'static + Sync + Send + Copy + Fn(&mut App, &CommandLineOptions, usize, Rank),
>(
    build_app: F,
    receivers: Receivers,
    senders: Senders,
    opts: &CommandLineOptions,
    rank: Rank,
) -> App {
    let mut app = App::new();
    app.insert_non_send_resource(receivers);
    app.insert_non_send_resource(senders);
    build_app(&mut app, opts, opts.num_threads, rank);
    app
}

pub fn build_local_communication_app<
    F: 'static + Sync + Copy + Send + Fn(&mut App, &CommandLineOptions, usize, Rank),
>(
    build_app: F,
) {
    use clap::Parser;

    let opts = CommandLineOptions::parse();
    let mut app = create_and_build_app(
        build_app,
        Receivers(HashMap::new()),
        Senders(HashMap::new()),
        &opts,
        0,
    );
    for rank in 1..opts.num_threads {
        let receivers = Receivers({
            let all = &mut app
                .world
                .get_non_send_resource_mut::<Receivers>()
                .unwrap()
                .0;
            let to_move = all
                .drain_filter(|comm, _| comm.owner == rank as Rank)
                .collect();
            to_move
        });
        let senders = Senders({
            let all = &mut app.world.get_non_send_resource_mut::<Senders>().unwrap().0;
            let to_move = all
                .drain_filter(|comm, _| comm.owner == rank as Rank)
                .collect();
            to_move
        });
        let opts = opts.clone();
        thread::spawn(move || {
            let mut app = create_and_build_app(build_app, receivers, senders, &opts, rank as Rank);
            app.run()
        });
    }
    app.run();
}

#[derive(PartialEq, Eq, Debug, Hash)]
struct Comm {
    owner: Rank,
    other: Rank,
    tag: Tag,
}

struct Receivers(HashMap<Comm, Receiver<Payload>>);

struct Senders(HashMap<Comm, Sender<Payload>>);

impl<T: Sync + Send + 'static> Plugin for CommunicationPlugin<T> {
    fn build(&self, app: &mut App) {
        let tag = {
            let mut tag = app.world.get_resource_mut::<CurrentTag>().unwrap();
            tag.0 += 1;
            tag.0
        };
        let rank = app.world.get_resource::<WorldRank>().unwrap().0;
        let world_size = app.world.get_resource::<NumRanks>().unwrap().0;
        let all_ranks = 0i32..world_size as i32;
        let other_ranks = (0i32..world_size as i32).filter(|r| *r != rank);
        if rank == 0 {
            for rank1 in all_ranks.clone() {
                for rank2 in all_ranks.clone() {
                    if rank1 == rank2 {
                        continue;
                    }
                    let (sender1, receiver1) = channel();
                    let (sender2, receiver2) = channel();
                    let mut receivers = app.world.get_non_send_resource_mut::<Receivers>().unwrap();
                    receivers.0.insert(
                        Comm {
                            owner: rank1,
                            other: rank2,
                            tag,
                        },
                        receiver1,
                    );
                    receivers.0.insert(
                        Comm {
                            owner: rank2,
                            other: rank1,
                            tag,
                        },
                        receiver2,
                    );
                    let mut senders = app.world.get_non_send_resource_mut::<Senders>().unwrap();
                    senders.0.insert(
                        Comm {
                            owner: rank2,
                            other: rank1,
                            tag,
                        },
                        sender1,
                    );
                    senders.0.insert(
                        Comm {
                            owner: rank1,
                            other: rank2,
                            tag,
                        },
                        sender2,
                    );
                }
            }
        }
        let mut commun = LocalCommunicator::<T>::new(
            DataByRank::empty(),
            DataByRank::empty(),
            tag,
            world_size,
            rank,
        );
        for r in other_ranks {
            if r == rank {
                continue;
            }
            let mut senders = app.world.get_non_send_resource_mut::<Senders>().unwrap();
            let sender = senders
                .0
                .remove(&Comm {
                    owner: rank,
                    other: r,
                    tag,
                })
                .unwrap();
            let mut receivers = app.world.get_non_send_resource_mut::<Receivers>().unwrap();
            let receiver = receivers
                .0
                .remove(&Comm {
                    owner: rank,
                    other: r,
                    tag,
                })
                .unwrap();
            commun.receivers.insert(r, receiver);
            commun.senders.insert(r, sender);
        }
        match self.type_ {
            CommunicationType::Exchange => {
                app.insert_non_send_resource(ExchangeCommunicator::from_communicator(commun));
            }
            CommunicationType::Sync => {
                todo!()
            }
            CommunicationType::Sum => {
                app.insert_non_send_resource(commun);
            }
            CommunicationType::AllGather => {
                app.insert_non_send_resource(commun);
            }
        }
    }
}
