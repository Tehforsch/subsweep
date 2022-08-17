use bevy::prelude::*;

use super::{spawn_sprites_system, position_to_translation_system};

pub struct RemoteVisualizationPlugin;

impl RemoteVisualizationPlugin {
    fn add_to_app(&self, app: &mut App) {
        // app.add_system(spawn_sprites_system)
            // .add_system(position_to_translation_system)
            // .add_system(send_particles_to_main_thread_system);
    }
}

// pub fn send_particles_to_main_thread_system(
//     mut commands: Commands,
//     particles: Query<(Entity, &Position, &Velocity)>,
//     mut communicator: NonSendMut<ExchangeCommunicator<C, ParticleExchangeData>>,
//     domain: Res<DomainDistribution>,
// ) where
//     C: Communicator<ParticleExchangeData>,
// {
// }
