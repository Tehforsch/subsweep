use std::marker::PhantomData;

use bevy::ecs::schedule::SystemLabelId;
use bevy::ecs::system::AsSystemLabel;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Entity;
use bevy::prelude::NonSendMut;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;

use super::DomainDecompositionStages;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::physics::LocalParticle;
use crate::plugin_utils::run_once;

#[derive(Default)]
struct ExchangePluginExists;

#[derive(Default)]
struct ExchangeOrder(Vec<SystemLabelId>);

#[derive(Default, Deref, DerefMut)]
pub(super) struct OutgoingEntities(DataByRank<Vec<Entity>>);

impl OutgoingEntities {
    pub fn add(&mut self, rank: Rank, entity: Entity) {
        self[rank].push(entity);
    }
}

#[derive(Default, Deref, DerefMut)]
struct SpawnedEntities(DataByRank<Vec<Entity>>);

#[derive(Deref, DerefMut)]
struct ExchangeBuffers<T>(DataByRank<Vec<T>>);

impl<T> ExchangeBuffers<T> {
    fn take(&mut self) -> DataByRank<Vec<T>> {
        std::mem::take(&mut self.0)
    }
}

pub struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

#[derive(Equivalence, Deref, DerefMut)]
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
        let rank = **app.world.get_resource::<WorldRank>().unwrap();
        let size = **app.world.get_resource::<WorldSize>().unwrap();
        run_once("exchange_data_plugin", app, |app| {
            app.insert_resource(OutgoingEntities(DataByRank::from_size_and_rank(size, rank)))
                .insert_resource(SpawnedEntities(DataByRank::from_size_and_rank(size, rank)))
                .insert_resource(ExchangeOrder::default())
                .add_plugin(CommunicationPlugin::<NumEntities>::new(
                    CommunicationType::Exchange,
                ))
                .add_system_to_stage(
                    DomainDecompositionStages::Exchange,
                    send_num_outgoing_entities_system,
                )
                .add_system_to_stage(
                    DomainDecompositionStages::Exchange,
                    despawn_outgoing_entities_system,
                )
                .add_system_to_stage(
                    DomainDecompositionStages::Exchange,
                    reset_outgoing_entities_system
                        .after(send_num_outgoing_entities_system)
                        .after(despawn_outgoing_entities_system),
                )
                .add_system_to_stage(
                    DomainDecompositionStages::Exchange,
                    spawn_incoming_entities_system.after(send_num_outgoing_entities_system),
                );
        });
        let labels = app.world.get_resource_mut::<ExchangeOrder>();
        let mut exchange_buffers_system = Self::exchange_buffers_system
            .after(Self::fill_buffers_system)
            .after(spawn_incoming_entities_system)
            .before(reset_outgoing_entities_system);
        for label in labels.as_ref().unwrap().0.iter() {
            exchange_buffers_system = exchange_buffers_system.after(*label);
        }
        let label = Self::exchange_buffers_system.as_system_label();
        labels.unwrap().0.push(label);
        app.insert_resource(ExchangeBuffers::<T>(DataByRank::from_size_and_rank(
            size, rank,
        )))
        .add_system_to_stage(DomainDecompositionStages::Exchange, exchange_buffers_system)
        .add_plugin(CommunicationPlugin::<T>::new(CommunicationType::Exchange))
        .add_system_to_stage(
            DomainDecompositionStages::Exchange,
            Self::fill_buffers_system,
        )
        .add_system_to_stage(
            DomainDecompositionStages::Exchange,
            Self::reset_buffers_system.after(Self::exchange_buffers_system),
        );
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> ExchangeDataPlugin<T> {
    fn fill_buffers_system(
        entity_exchange: Res<OutgoingEntities>,
        query: Query<&T>,
        mut buffer: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, entities) in entity_exchange.iter() {
            // This allocates a new buffer every time. An alternative would be
            // to keep this at maximum size, trading performance for memory overhead
            buffer.insert(
                *rank,
                entities
                    .iter()
                    .map(|entity| query.get(*entity).unwrap().clone())
                    .collect(),
            );
        }
    }

    fn exchange_buffers_system(
        mut commands: Commands,
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        mut buffers: ResMut<ExchangeBuffers<T>>,
        spawned_entities: Res<SpawnedEntities>,
    ) {
        let buffers = buffers.take();
        let mut incoming = communicator.exchange_all(buffers);
        for (rank, data) in incoming.drain_all() {
            let spawned_entities = spawned_entities[rank].clone();
            for (entity, component) in spawned_entities.iter().zip(data.into_iter()) {
                commands.entity(*entity).insert(component);
            }
        }
    }

    fn reset_buffers_system(
        mut buffers: ResMut<ExchangeBuffers<T>>,
        size: Res<WorldSize>,
        rank: Res<WorldRank>,
    ) {
        *buffers = ExchangeBuffers(DataByRank::from_size_and_rank(**size, **rank));
    }
}

fn send_num_outgoing_entities_system(
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    num_outgoing: Res<OutgoingEntities>,
) {
    for rank in communicator.other_ranks() {
        communicator.send(rank, NumEntities(num_outgoing.get(&rank).unwrap().len()));
    }
}

fn spawn_incoming_entities_system(
    mut commands: Commands,
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    mut spawned_entities: ResMut<SpawnedEntities>,
) {
    for (rank, num_incoming) in communicator.receive() {
        spawned_entities.insert(
            rank,
            (0..*num_incoming)
                .map(|_| {
                    let id = commands.spawn().insert(LocalParticle).id();
                    id
                })
                .collect(),
        );
    }
}

fn reset_outgoing_entities_system(
    mut outgoing: ResMut<OutgoingEntities>,
    size: Res<WorldSize>,
    rank: Res<WorldRank>,
) {
    *outgoing = OutgoingEntities(DataByRank::from_size_and_rank(**size, **rank));
}

fn despawn_outgoing_entities_system(
    mut commands: Commands,
    entity_exchange: Res<OutgoingEntities>,
) {
    for (_, entities) in entity_exchange.iter() {
        for entity in entities {
            commands.entity(*entity).despawn();
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "mpi"))]
mod tests {
    use bevy::prelude::App;
    use bevy::prelude::Component;
    use bevy::prelude::CoreStage;
    use bevy::prelude::SystemStage;
    use mpi::traits::Equivalence;

    use crate::communication::build_local_communication_app_with_custom_logic;
    use crate::communication::WorldRank;
    use crate::domain::exchange_data_plugin::ExchangeDataPlugin;
    use crate::domain::exchange_data_plugin::OutgoingEntities;
    use crate::domain::DomainDecompositionStages;

    #[derive(Clone, Equivalence, Component)]
    struct A {
        x: i32,
        y: f64,
    }
    #[derive(Clone, Equivalence, Component)]
    struct B {
        x: i64,
        y: bool,
    }

    fn check_received(mut app: App) {
        let is_main = app.world.get_resource::<WorldRank>().unwrap().is_main();
        let mut entities = vec![];
        if is_main {
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 0, y: 5.0 })
                    .insert(B { x: 0, y: false })
                    .id(),
            );
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 1, y: 10.0 })
                    .insert(B { x: 1, y: true })
                    .id(),
            );
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 2, y: 20.0 })
                    .insert(B { x: 2, y: false })
                    .id(),
            );
        }
        let check_num_entities = |app: &mut App, rank_0_count: usize, rank_1_count: usize| {
            let mut query = app.world.query::<&mut A>();
            let count = query.iter(&app.world).count();
            if is_main {
                assert_eq!(count, rank_0_count);
            } else {
                assert_eq!(count, rank_1_count);
            }
        };
        let mut exchange_first_entity = |app: &mut App| {
            if is_main {
                let mut outgoing = app.world.get_resource_mut::<OutgoingEntities>().unwrap();
                outgoing.add(1, entities.remove(0));
            }
        };
        check_num_entities(&mut app, 3, 0);
        exchange_first_entity(&mut app);
        app.update();
        check_num_entities(&mut app, 2, 1);
        app.update();
        check_num_entities(&mut app, 2, 1);
        exchange_first_entity(&mut app);
        exchange_first_entity(&mut app);
        app.update();
        check_num_entities(&mut app, 0, 3);
    }

    #[test]
    fn exchange_data_plugin() {
        build_local_communication_app_with_custom_logic(build_app, check_received, 2);
    }

    fn build_app(app: &mut App) {
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Exchange,
            SystemStage::parallel(),
        )
        .add_plugin(ExchangeDataPlugin::<A>::default())
        .add_plugin(ExchangeDataPlugin::<B>::default());
    }
}
