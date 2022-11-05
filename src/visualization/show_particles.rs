use bevy::prelude::*;

use super::color::color_map;
use super::draw_item::draw_translation_system;
use super::draw_item::DrawItem;
use super::DrawCircle;
use super::VisualizationParameters;
use super::VisualizationStage;
use crate::components::Position;
use crate::named::Named;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::WorldRank;
use crate::simulation::RaxiomPlugin;

#[derive(Named)]
pub struct ShowParticlesPlugin;

impl RaxiomPlugin for ShowParticlesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(VisualizationStage::AddVisualization, show_particles_system)
            .add_system_to_stage(
                VisualizationStage::Draw,
                position_to_translation_system.before(draw_translation_system::<DrawCircle>),
            );
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.unwrap_resource::<VisualizationParameters>()
            .show_particles
    }
}

fn show_particles_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position), Without<DrawCircle>>,
    rank: Res<WorldRank>,
) {
    for (entity, pos) in particles.iter() {
        commands
            .entity(entity)
            .insert(DrawCircle::from_position_and_color(
                **pos,
                color_map(**rank),
            ));
    }
}

fn position_to_translation_system(mut query: Particles<(&mut DrawCircle, &Position)>) {
    for (mut item, position) in query.iter_mut() {
        item.set_translation(position);
    }
}
