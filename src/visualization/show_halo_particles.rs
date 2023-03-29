use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::IntoSystemDescriptor;
use bevy::prelude::Without;

use super::draw_item::draw_translation_system;
use super::VisualizationParameters;
use crate::components::Position;
use crate::named::Named;
use crate::particle::HaloParticles;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;
use crate::visualization::DrawCircle;
use crate::visualization::DrawItem;
use crate::visualization::Pixels;
use crate::visualization::RColor;
use crate::visualization::VisualizationStage;

#[derive(Named)]
pub struct ShowHaloParticlesPlugin;

impl RaxiomPlugin for ShowHaloParticlesPlugin {
    fn should_build(&self, sim: &Simulation) -> bool {
        sim.unwrap_resource::<VisualizationParameters>()
            .show_halo_particles
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(
            VisualizationStage::AddVisualization,
            show_halo_particles_system,
        )
        .add_system_to_stage(
            VisualizationStage::DrawOnMainRank,
            position_to_translation_system.before(draw_translation_system::<DrawCircle>),
        );
    }
}

fn show_halo_particles_system(
    mut commands: Commands,
    undrawn_halo_particles: HaloParticles<(Entity, &Position), Without<DrawCircle>>,
) {
    for (entity, pos) in undrawn_halo_particles.iter() {
        commands.entity(entity).insert(DrawCircle {
            position: **pos,
            radius: Pixels(10.0),
            color: RColor::RED,
        });
    }
}

fn position_to_translation_system(mut query: HaloParticles<(&mut DrawCircle, &Position)>) {
    for (mut item, position) in query.iter_mut() {
        item.set_translation(position);
    }
}
