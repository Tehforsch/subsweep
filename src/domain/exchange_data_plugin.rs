use std::marker::PhantomData;

use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::NonSendMut;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::SystemSet;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;

use super::DomainDecompositionStages;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::SizedCommunicator;

struct ExchangePluginExists;

#[derive(Default)]
struct OutgoingEntities(DataByRank<Vec<Entity>>);

#[derive(Default)]
struct SpawnedEntities(DataByRank<Vec<Entity>>);

struct ExchangeBuffers<T>(DataByRank<Vec<T>>);

struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

#[derive(Equivalence)]
struct NumEntities(usize);

impl<T> Default for ExchangeDataPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> Plugin for ExchangeDataPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        let exists = app.world.get_resource_mut::<ExchangePluginExists>();
        let first = exists.is_none();
        if first {
            app.insert_resource(ExchangePluginExists)
                .insert_resource(OutgoingEntities::default())
                .insert_resource(SpawnedEntities::default())
                .add_plugin(CommunicationPlugin::<NumEntities>::new(
                    CommunicationType::Exchange,
                ));
        }
        app.insert_resource(ExchangeBuffers::<T>(DataByRank::default()))
            .add_plugin(CommunicationPlugin::<T>::new(CommunicationType::Exchange))
            .add_system_set_to_stage(
                DomainDecompositionStages::Exchange,
                SystemSet::new()
                    .with_system(Self::fill_buffers_system)
                    .with_system(Self::despawn_outgoing_entities_system)
                    .with_system(Self::send_buffers_system.after(Self::fill_buffers_system))
                    .with_system(
                        Self::receive_buffers_system
                            .after(Self::send_buffers_system)
                            .after(spawn_incoming_entities_system),
                    ),
            );
        if first {
            app.add_system_to_stage(
                DomainDecompositionStages::Exchange,
                send_num_outgoing_entities_system,
            )
            .add_system_to_stage(
                DomainDecompositionStages::Exchange,
                reset_outgoing_entities_system,
            )
            .add_system_to_stage(
                DomainDecompositionStages::Exchange,
                spawn_incoming_entities_system.after(send_num_outgoing_entities_system),
            );
        }
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> ExchangeDataPlugin<T> {
    fn fill_buffers_system(
        entity_exchange: Res<OutgoingEntities>,
        query: Query<&T>,
        mut buffer: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, entities) in entity_exchange.0.iter() {
            // This allocates a new buffer every time. An alternative would be
            // to keep this at maximum size, trading performance for memory overhead
            buffer.0.insert(
                *rank,
                entities
                    .iter()
                    .enumerate()
                    .map(|(i, entity)| query.get(*entity).unwrap().clone())
                    .collect(),
            );
        }
    }

    fn despawn_outgoing_entities_system(
        mut commands: Commands,
        entity_exchange: Res<OutgoingEntities>,
    ) {
        for (_, entities) in entity_exchange.0.iter() {
            for entity in entities {
                commands.entity(*entity).despawn();
            }
        }
    }

    fn send_buffers_system(
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        mut buffers: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, data) in buffers.0.drain_all() {
            communicator.send_vec(rank, data);
        }
    }

    fn receive_buffers_system(
        mut commands: Commands,
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        spawned_entities: Res<SpawnedEntities>,
    ) {
        for (rank, data) in communicator.receive_vec() {
            let spawned_entities = spawned_entities.0[rank].clone();
            let iterator = spawned_entities
                .into_iter()
                .zip(data.into_iter().map(|component| (component,)));
            commands.insert_or_spawn_batch(iterator);
        }
    }
}

fn send_num_outgoing_entities_system(
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    num_outgoing: Res<OutgoingEntities>,
) {
    for rank in communicator.other_ranks() {
        communicator.send(
            rank,
            NumEntities(
                num_outgoing
                    .0
                    .get(&rank)
                    .map(|data| data.len())
                    .unwrap_or(0),
            ),
        );
    }
}

fn spawn_incoming_entities_system(
    mut commands: Commands,
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    mut spawned_entities: ResMut<SpawnedEntities>,
) {
    for (rank, num_incoming) in communicator.receive() {
        spawned_entities.0.insert(
            rank,
            (0..num_incoming.0)
                .map(|_| {
                    let id = commands.spawn().id();
                    id
                })
                .collect(),
        );
    }
}

fn reset_outgoing_entities_system(mut outgoing: ResMut<OutgoingEntities>) {
    *outgoing = OutgoingEntities::default();
}

#[cfg(test)]
#[cfg(feature = "local")]
mod tests {
    use bevy::prelude::App;
    use bevy::prelude::Component;
    use bevy::prelude::CoreStage;
    use bevy::prelude::SystemStage;
    use mpi::traits::Equivalence;

    use crate::communication::build_local_communication_app_with_custom_logic;
    use crate::communication::BaseCommunicationPlugin;
    use crate::communication::WorldRank;
    use crate::domain::exchange_data_plugin::ExchangeDataPlugin;
    use crate::domain::exchange_data_plugin::OutgoingEntities;
    use crate::domain::DomainDecompositionStages;

    #[derive(Clone, Equivalence, Component)]
    struct A {
        x: i32,
        y: f32,
    }
    #[derive(Clone, Equivalence, Component)]
    struct B {
        x: i64,
        y: bool,
    }

    fn check_received(mut app: App) {
        let is_main = app.world.get_resource::<WorldRank>().unwrap().is_main();
        app.update();
        if !is_main {
            let mut query = app.world.query::<&mut A>();
            assert_eq!(query.iter(&app.world).count(), 1);
        }
        app.update();
        if !is_main {
            let mut query = app.world.query::<&mut A>();
            assert_eq!(query.iter(&app.world).count(), 1);
        }
    }

    #[test]
    fn exchange_data_plugin() {
        build_local_communication_app_with_custom_logic(
            |app, size, rank| build_app(app, size, rank),
            check_received,
            2,
        );
    }

    fn build_app(app: &mut App, size: usize, rank: i32) {
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Exchange,
            SystemStage::parallel(),
        )
        .add_plugin(BaseCommunicationPlugin::new(size, rank))
        .add_plugin(ExchangeDataPlugin::<A>::default())
        .add_plugin(ExchangeDataPlugin::<B>::default());
        if rank == 0 {
            let mut entities = vec![];
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 0, y: 5.0 })
                    .insert(B { x: 0, y: false })
                    .id(),
            );
            let mut outgoing = app.world.get_resource_mut::<OutgoingEntities>().unwrap();
            outgoing.0.insert(1, entities);
        }
    }
}
