use bevy::prelude::*;
use mpi::traits::Equivalence;

use crate::{communication::ExchangeCommunicator, position::Position};

use super::{spawn_sprites_system, position_to_translation_system};

pub struct RemoteVisualizationPlugin;

#[derive(Equivalence)]
struct ParticleVisualizationExchangeData {
    pos: Position,
}

impl RemoteVisualizationPlugin {
    fn add_to_app(&self, app: &mut App) {
        app
            .add_system(send_particles_to_main_thread_system);
    }
}

fn send_particles_to_main_thread_system(
    mut commands: Commands,
    mut communicator: NonSendMut<ExchangeCommunicator<ParticleVisualizationExchangeData>>,
) 
{
}
