use std::marker::PhantomData;

use bevy::ecs::schedule::SystemLabelId;
use bevy::ecs::system::AsSystemLabel;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Entity;
use bevy::prelude::ParallelSystemDescriptorCoercion;
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
use crate::named::Named;
use crate::physics::LocalParticle;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

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

#[derive(Named)]
pub struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for ExchangeDataPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

#[derive(Equivalence, Deref, DerefMut)]
struct NumEntities(usize);

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> RaxiomPlugin
    for ExchangeDataPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        let rank = **sim.unwrap_resource::<WorldRank>();
        let size = **sim.unwrap_resource::<WorldSize>();
        sim.insert_resource(OutgoingEntities(DataByRank::from_size_and_rank(size, rank)))
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
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        let rank = **sim.unwrap_resource::<WorldRank>();
        let size = **sim.unwrap_resource::<WorldSize>();
        let labels = sim.get_resource_mut::<ExchangeOrder>();
        let mut exchange_buffers_system = Self::exchange_buffers_system
            .after(Self::fill_buffers_system)
            .after(spawn_incoming_entities_system)
            .before(reset_outgoing_entities_system);
        for label in labels.as_ref().unwrap().0.iter() {
            exchange_buffers_system = exchange_buffers_system.after(*label);
        }
        let label = Self::exchange_buffers_system.as_system_label();
        labels.unwrap().0.push(label);
        sim.insert_resource(ExchangeBuffers::<T>(DataByRank::from_size_and_rank(
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
        mut communicator: ExchangeCommunicator<T>,
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
    mut communicator: ExchangeCommunicator<NumEntities>,
    num_outgoing: Res<OutgoingEntities>,
) {
    for rank in communicator.other_ranks() {
        communicator.send(rank, NumEntities(num_outgoing.get(&rank).unwrap().len()));
    }
}

fn spawn_incoming_entities_system(
    mut commands: Commands,
    mut communicator: ExchangeCommunicator<NumEntities>,
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
    use bevy::prelude::Component;
    use mpi::traits::Equivalence;

    use crate::communication::build_local_communication_sim_with_custom_logic;
    use crate::communication::WorldRank;
    use crate::domain::exchange_data_plugin::ExchangeDataPlugin;
    use crate::domain::exchange_data_plugin::OutgoingEntities;
    use crate::simulation::Simulation;
    use crate::stages::SimulationStagesPlugin;

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

    fn check_received(mut sim: Simulation) {
        let is_main = sim.unwrap_resource::<WorldRank>().is_main();
        let mut entities = vec![];
        if is_main {
            entities.push(
                sim.world()
                    .spawn()
                    .insert(A { x: 0, y: 5.0 })
                    .insert(B { x: 0, y: false })
                    .id(),
            );
            entities.push(
                sim.world()
                    .spawn()
                    .insert(A { x: 1, y: 10.0 })
                    .insert(B { x: 1, y: true })
                    .id(),
            );
            entities.push(
                sim.world()
                    .spawn()
                    .insert(A { x: 2, y: 20.0 })
                    .insert(B { x: 2, y: false })
                    .id(),
            );
        }
        let check_num_entities =
            |sim: &mut Simulation, rank_0_count: usize, rank_1_count: usize| {
                let mut query = sim.world().query::<&mut A>();
                let count = query.iter(&sim.world()).count();
                if is_main {
                    assert_eq!(count, rank_0_count);
                } else {
                    assert_eq!(count, rank_1_count);
                }
            };
        let mut exchange_first_entity = |sim: &mut Simulation| {
            if is_main {
                let mut outgoing = sim.unwrap_resource_mut::<OutgoingEntities>();
                outgoing.add(1, entities.remove(0));
            }
        };
        check_num_entities(&mut sim, 3, 0);
        exchange_first_entity(&mut sim);
        sim.update();
        check_num_entities(&mut sim, 2, 1);
        sim.update();
        check_num_entities(&mut sim, 2, 1);
        exchange_first_entity(&mut sim);
        exchange_first_entity(&mut sim);
        sim.update();
        check_num_entities(&mut sim, 0, 3);
    }

    #[test]
    fn exchange_data_plugin() {
        build_local_communication_sim_with_custom_logic(build_sim, check_received, 2);
    }

    fn build_sim(sim: &mut Simulation) {
        sim.add_plugin(SimulationStagesPlugin)
            .add_plugin(ExchangeDataPlugin::<A>::default())
            .add_plugin(ExchangeDataPlugin::<B>::default());
    }
}
