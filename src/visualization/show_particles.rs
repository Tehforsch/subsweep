use bevy::prelude::*;
use derive_custom::raxiom_parameters;

use super::color::color_map;
use super::draw_item::change_colors_system;
use super::draw_item::draw_translation_system;
use super::draw_item::insert_meshes_system;
use super::draw_item::DrawItem;
use super::DrawCircle;
use super::RColor;
use super::VisualizationParameters;
use super::VisualizationStage;
use crate::components::InternalEnergy;
use crate::components::Mass;
use crate::components::Position;
use crate::components::Pressure;
use crate::components::SmoothingLength;
use crate::named::Named;
use crate::prelude::Float;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::WorldRank;
use crate::simulation::RaxiomPlugin;
use crate::simulation_box::SimulationBox;
use crate::units;
use crate::units::Dimensionless;
use crate::units::EnergyPerMass;
use crate::units::Temperature;

// The molecular weight that this plugin just blindly assumes.
const MOLECULAR_WEIGHT: Float = 4.0;

/// Which quantity is shown via the particle color.
#[derive(Default)]
#[raxiom_parameters]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ColorMap {
    /// Show the rank to which the particle belongs (default).
    #[default]
    Rank,
    /// Show the particle temperature (only available if hydrodynamics
    /// is enabled)
    Temperature {
        scale: Temperature,
    },
    Pressure {
        scale: units::Pressure,
    },
    Mass {
        scale: units::Mass,
    },
    NumParticles {
        scale: usize,
    },
}

#[derive(Named)]
pub struct ShowParticlesPlugin;

impl RaxiomPlugin for ShowParticlesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        todo!("all of the lower systems are ambiguous");
        sim.add_system_to_stage(VisualizationStage::AddVisualization, show_particles_system)
            .add_system_to_stage(
                VisualizationStage::Draw,
                position_to_translation_system
                    .before(draw_translation_system::<DrawCircle>)
                    .after(change_colors_system::<DrawCircle>),
            )
            .add_system_set_to_stage(
                VisualizationStage::AddDrawComponents,
                SystemSet::new()
                    .with_system(color_particles_by_temperature_system)
                    .with_system(color_particles_by_mass_system)
                    .with_system(color_particles_by_pressure_system)
                    .with_system(color_particles_by_num_particles_system)
                    .after(insert_meshes_system::<DrawCircle>),
            );
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.unwrap_resource::<VisualizationParameters>()
            .show_particles
    }
}

fn temperature_color_map(e: EnergyPerMass, scale: Temperature) -> RColor {
    RColor::reds((e.to_temperature(Dimensionless::dimensionless(MOLECULAR_WEIGHT)) / scale).value())
}

fn color_particles_by_temperature_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &InternalEnergy, &Mass)>,
) {
    if let ColorMap::Temperature { scale } = visualization_parameters.color_map {
        for (mut circle, internal_energy, mass) in particles.iter_mut() {
            circle.color = temperature_color_map(**internal_energy / **mass, scale);
        }
    }
}

fn color_particles_by_pressure_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &Pressure)>,
) {
    if let ColorMap::Pressure { scale } = visualization_parameters.color_map {
        for (mut circle, pressure) in particles.iter_mut() {
            circle.color = RColor::reds((**pressure / scale).value());
        }
    }
}

fn color_particles_by_num_particles_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &Position, &SmoothingLength)>,
    tree: Res<crate::hydrodynamics::QuadTree>,
    box_size: Res<SimulationBox>,
) {
    if let ColorMap::NumParticles { scale } = visualization_parameters.color_map {
        for (mut circle, pos, l) in particles.iter_mut() {
            let num_neighbours = tree.get_particles_in_radius(&box_size, pos, l).len();
            circle.color = RColor::reds(num_neighbours as f64 / scale as f64);
        }
    }
}

fn color_particles_by_mass_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &Mass)>,
) {
    if let ColorMap::Mass { scale } = visualization_parameters.color_map {
        for (mut circle, mass) in particles.iter_mut() {
            circle.color = RColor::reds((**mass / scale).value());
        }
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
