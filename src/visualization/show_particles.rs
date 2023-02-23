use bevy::prelude::*;
use derive_custom::raxiom_parameters;
use ordered_float::OrderedFloat;

use super::color::color_map;
use super::draw_item::change_colors_system;
use super::draw_item::draw_translation_system;
use super::draw_item::DrawItem;
use super::DrawCircle;
use super::RColor;
use super::VisualizationParameters;
use super::VisualizationStage;
use crate::components::Density;
use crate::components::InternalEnergy;
use crate::components::IonizedHydrogenFraction;
use crate::components::Mass;
use crate::components::Position;
use crate::components::Pressure;
use crate::named::Named;
use crate::parameters::SimulationBox;
use crate::parameters::SweepParameters;
use crate::prelude::Float;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::WorldRank;
use crate::simulation::RaxiomPlugin;
use crate::sweep::timestep_level::TimestepLevel;
use crate::units::Dimension;
use crate::units::Dimensionless;
use crate::units::EnergyPerMass;
use crate::units::Quantity;
use crate::units::Temperature;
use crate::units::DENSITY;
use crate::units::DIMENSIONLESS;
use crate::units::MASS;
use crate::units::PRESSURE;
use crate::units::TEMPERATURE;

// The molecular weight that this plugin just blindly assumes.
const MOLECULAR_WEIGHT: Float = 4.0;

#[raxiom_parameters]
#[derive(Default)]
pub enum Scale<const D: Dimension> {
    #[default]
    Auto,
    Explicit(Quantity<Float, D>),
}

impl<const D: Dimension> Scale<D> {
    fn to_value<'a>(&self, values: impl Iterator<Item = Quantity<Float, D>>) -> Quantity<Float, D> {
        match self {
            Scale::Auto => values
                .max_by_key(|x| OrderedFloat(x.value_unchecked()))
                .unwrap_or(Quantity::<Float, D>::zero()),
            Scale::Explicit(value) => *value,
        }
    }
}

/// Which quantity is shown via the particle color.
#[derive(Default)]
#[raxiom_parameters]
#[serde(tag = "type")]
pub enum ColorMap {
    /// Show the rank to which the particle belongs (default).
    #[default]
    Rank,
    /// Show the particle temperature (only available if hydrodynamics
    /// is enabled)
    Temperature {
        scale: Scale<TEMPERATURE>,
    },
    Density {
        scale: Scale<DENSITY>,
    },
    Pressure {
        scale: Scale<PRESSURE>,
    },
    Mass {
        scale: Scale<MASS>,
    },
    IonizedHydrogenFraction {
        scale: Scale<DIMENSIONLESS>,
    },
    TimestepLevel {},
}

#[derive(Named)]
pub struct ShowParticlesPlugin;

#[derive(SystemLabel)]
struct ColorParticlesLabel;

impl RaxiomPlugin for ShowParticlesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(VisualizationStage::AddVisualization, show_particles_system)
            .add_system_to_stage(
                VisualizationStage::DrawOnMainRank,
                position_to_translation_system
                    .before(draw_translation_system::<DrawCircle>)
                    .after(change_colors_system::<DrawCircle>),
            )
            .add_system_set_to_stage(
                VisualizationStage::ModifyVisualization,
                SystemSet::new()
                    .with_system(
                        color_particles_by_temperature_system.ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(color_particles_by_mass_system.ambiguous_with(ColorParticlesLabel))
                    .with_system(
                        color_particles_by_ionized_hydrogen_fraction_system
                            .ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(
                        color_particles_by_timestep_level_system
                            .ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(
                        color_particles_by_pressure_system.ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(
                        color_particles_by_density_system.ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(change_color_map_system)
                    .label(ColorParticlesLabel),
            );
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.unwrap_resource::<VisualizationParameters>()
            .show_particles
    }
}

fn get_temperature(e: EnergyPerMass) -> Temperature {
    e.to_temperature(Dimensionless::dimensionless(MOLECULAR_WEIGHT))
}
fn temperature_color_map(e: EnergyPerMass, scale: Temperature) -> RColor {
    RColor::reds((e.to_temperature(Dimensionless::dimensionless(MOLECULAR_WEIGHT)) / scale).value())
}

fn color_particles_by_temperature_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &InternalEnergy, &Mass)>,
) {
    if let ColorMap::Temperature { ref scale } = visualization_parameters.color_map {
        let scale = scale.to_value(
            particles
                .iter()
                .map(|(_, internal_energy, mass)| get_temperature(**internal_energy / **mass)),
        );
        for (mut circle, internal_energy, mass) in particles.iter_mut() {
            circle.color = temperature_color_map(**internal_energy / **mass, scale);
        }
    }
}

fn color_particles_by_density_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &Density)>,
) {
    if let ColorMap::Density { ref scale } = visualization_parameters.color_map {
        let scale = scale.to_value(particles.iter().map(|(_, density)| **density));
        for (mut circle, density) in particles.iter_mut() {
            circle.color = RColor::reds((**density / scale).value());
        }
    }
}

fn color_particles_by_pressure_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &Pressure)>,
) {
    if let ColorMap::Pressure { ref scale } = visualization_parameters.color_map {
        let scale = scale.to_value(particles.iter().map(|(_, pressure)| **pressure));
        for (mut circle, pressure) in particles.iter_mut() {
            circle.color = RColor::reds((**pressure / scale).value());
        }
    }
}

fn color_particles_by_mass_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &Mass)>,
) {
    if let ColorMap::Mass { ref scale } = visualization_parameters.color_map {
        let scale = scale.to_value(particles.iter().map(|(_, mass)| **mass));
        for (mut circle, mass) in particles.iter_mut() {
            circle.color = RColor::reds((**mass / scale).value());
        }
    }
}

fn color_particles_by_ionized_hydrogen_fraction_system(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &IonizedHydrogenFraction)>,
) {
    if let ColorMap::IonizedHydrogenFraction { ref scale } = visualization_parameters.color_map {
        let scale = scale.to_value(particles.iter().map(|(_, fraction)| **fraction));
        for (mut circle, fraction) in particles.iter_mut() {
            circle.color = RColor::reds((**fraction / scale).value());
        }
    }
}

fn color_particles_by_timestep_level_system(
    visualization_parameters: Res<VisualizationParameters>,
    sweep_parameters: Res<SweepParameters>,
    mut particles: Particles<(&mut DrawCircle, &TimestepLevel)>,
) {
    if let ColorMap::TimestepLevel {} = visualization_parameters.color_map {
        for (mut circle, level) in particles.iter_mut() {
            circle.color = RColor::reds(
                level.0 as Float / (sweep_parameters.num_timestep_levels - 1) as Float,
            );
        }
    }
}

fn show_particles_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position), Without<DrawCircle>>,
    rank: Res<WorldRank>,
    visualization_parameters: Res<VisualizationParameters>,
    box_size: Res<SimulationBox>,
) {
    for (entity, pos) in particles.iter() {
        if visualization_parameters.slice.contains(**pos, &box_size) {
            commands
                .entity(entity)
                .insert(DrawCircle::from_position_and_color(
                    **pos,
                    color_map(**rank),
                ));
        }
    }
}

fn position_to_translation_system(mut query: Particles<(&mut DrawCircle, &Position)>) {
    for (mut item, position) in query.iter_mut() {
        item.set_translation(position);
    }
}

fn change_color_map_system(
    mut parameters: ResMut<VisualizationParameters>,
    input: Res<Input<KeyCode>>,
) {
    for key in input.get_just_pressed() {
        match key {
            KeyCode::I => {
                parameters.color_map = ColorMap::IonizedHydrogenFraction { scale: Scale::Auto }
            }
            KeyCode::M => parameters.color_map = ColorMap::Mass { scale: Scale::Auto },
            KeyCode::D => parameters.color_map = ColorMap::Density { scale: Scale::Auto },
            KeyCode::L => parameters.color_map = ColorMap::TimestepLevel {},
            KeyCode::T => parameters.color_map = ColorMap::Temperature { scale: Scale::Auto },
            _ => {}
        }
    }
}
