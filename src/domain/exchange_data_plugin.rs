use std::marker::PhantomData;

use bevy::prelude::Entity;
use bevy::prelude::EventReader;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::ResMut;
use bevy::prelude::SystemSet;

use super::DomainDecompositionStages;
use crate::communication::Rank;

struct ExchangePluginExists;

struct EntityExchangedEvent {
    entity: Entity,
    target_rank: Rank,
}

struct ExchangeBuffer<T>(Vec<T>);

struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for ExchangeDataPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Sync + Send + 'static> Plugin for ExchangeDataPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        let exists = app.world.get_resource_mut::<ExchangePluginExists>();
        let first = exists.is_none();
        if first {
            app.world.insert_resource(ExchangePluginExists);
        }
        app.add_system_set_to_stage(
            DomainDecompositionStages::Exchange,
            SystemSet::new()
                .with_system(Self::prepare_buffers_system)
                .with_system(Self::fill_buffers_system.after(Self::prepare_buffers_system))
                .with_system(
                    Self::despawn_outgoing_entities_system.after(Self::prepare_buffers_system),
                )
                .with_system(Self::send_buffers_system.after(Self::fill_buffers_system))
                .with_system(Self::receive_buffers_system.after(Self::send_buffers_system))
                .with_system(
                    Self::insert_incoming_components_system.after(spawn_incoming_entities_system),
                ),
        );
        if first {
            app.add_system_to_stage(
                DomainDecompositionStages::Exchange,
                spawn_incoming_entities_system.after(Self::receive_buffers_system),
            );
        }
    }
}

impl<T: Sync + Send + 'static> ExchangeDataPlugin<T> {
    fn prepare_buffers_system(
        events: EventReader<EntityExchangedEvent>,
        mut buffer: ResMut<ExchangeBuffer<T>>,
    ) {
        let num_exchanged = events.len();
        // This allocates a new buffer every time. An alternative would be
        // to keep this at maximum size, trading performance for memory overhead
        buffer.0 = Vec::with_capacity(num_exchanged);
    }

    fn fill_buffers_system(
        events: EventReader<EntityExchangedEvent>,
        mut buffer: ResMut<ExchangeBuffer<T>>,
    ) {
        todo!()
    }

    fn despawn_outgoing_entities_system(
        events: EventReader<EntityExchangedEvent>,
        mut buffer: ResMut<ExchangeBuffer<T>>,
    ) {
        todo!()
    }

    fn send_buffers_system(
        events: EventReader<EntityExchangedEvent>,
        mut buffer: ResMut<ExchangeBuffer<T>>,
    ) {
        todo!()
    }

    fn receive_buffers_system(
        events: EventReader<EntityExchangedEvent>,
        mut buffer: ResMut<ExchangeBuffer<T>>,
    ) {
        todo!()
    }

    fn insert_incoming_components_system(
        events: EventReader<EntityExchangedEvent>,
        mut buffer: ResMut<ExchangeBuffer<T>>,
    ) {
        todo!()
    }
}

fn spawn_incoming_entities_system(events: EventReader<EntityExchangedEvent>) {
    todo!()
}
