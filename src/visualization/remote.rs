use bevy::prelude::*;
use mpi::traits::Equivalence;

use super::draw_item::DrawItem;
use crate::communication::Rank;
use crate::communication::SyncCommunicator;
use crate::communication::WorldRank;

#[derive(Component)]
pub struct RemoteDrawItem;

#[derive(Bundle)]
pub struct RemoteItemBundle<T: Component + Sync + Send + 'static> {
    data: T,
    marker: RemoteDrawItem,
}

pub(super) fn send_items_to_main_thread_system<T: Clone + Component + Equivalence + DrawItem>(
    rank: Res<WorldRank>,
    mut communicator: SyncCommunicator<T>,
    particles: Query<(Entity, &T)>,
) {
    debug_assert!(!rank.is_main());
    for (entity, item) in particles.iter() {
        communicator.send_sync(WorldRank::main(), entity, item.clone());
    }
    let _ = communicator.receive_sync(|_, _| panic!("No items expected"));
}

pub(super) fn receive_items_on_main_thread_system<T: Clone + Component + Equivalence + DrawItem>(
    mut commands: Commands,
    rank: Res<WorldRank>,
    mut communicator: SyncCommunicator<T>,
    mut particles: Query<&mut T, With<RemoteDrawItem>>,
) {
    debug_assert!(rank.is_main());
    let spawn_particle = |_: Rank, data: T| {
        commands
            .spawn()
            .insert_bundle(RemoteItemBundle::<T> {
                data,
                marker: RemoteDrawItem,
            })
            .id()
    };
    let mut sync = communicator.receive_sync(spawn_particle);
    sync.despawn_deleted(&mut commands);
    for (_, data) in sync.updated.drain_all() {
        for (entity, new_data) in data.into_iter() {
            *particles.get_mut(entity).unwrap() = new_data;
        }
    }
}
