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

// The molecular weight that this plugin just blindly assumes.
const MOLECULAR_WEIGHT: Float = 4.0;

#[raxiom_parameters]
#[derive(Default)]
pub enum Scale<const D: Dimension> {
    #[default]
    Auto,
    Explicit(Quantity<Float, D>),
}

fn get_scale<const D: Dimension>(
    values: impl Iterator<Item = Quantity<Float, D>>,
) -> Quantity<Float, D> {
    values
        .max_by_key(|x| OrderedFloat(x.value_unchecked()))
        .unwrap_or(Quantity::<Float, D>::zero())
}

/// Which quantity is shown via the particle color.
#[derive(Default, Copy, PartialEq, Eq)]
#[raxiom_parameters]
#[serde(tag = "type")]
pub enum ColorMap {
    #[default]
    Rank,
    Density,
    Temperature,
    Mass,
    IonizedHydrogenFraction,
    TimestepLevel,
}

impl ColorMap {
    fn map(&self) -> &dyn Fn(f64) -> RColor {
        match self {
            ColorMap::Rank => &RColor::reds,
            ColorMap::Density => &RColor::blues,
            ColorMap::Temperature => &RColor::reds,
            ColorMap::Mass => &RColor::reds,
            ColorMap::IonizedHydrogenFraction => &RColor::greens,
            ColorMap::TimestepLevel => &RColor::greys,
        }
    }
}

#[derive(Named)]
pub struct ShowParticlesPlugin;

#[derive(SystemLabel)]
struct ColorParticlesLabel;

impl RaxiomPlugin for ShowParticlesPlugin {
    fn build_on_main_rank(&self, sim: &mut Simulation) {
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
                    .with_system(
                        color_particles_by_timestep_level_system
                            .ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(color_particles_by_quantity_system::<Mass, { ColorMap::Mass }, MASS>
                            .ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(color_particles_by_quantity_system::<Density, { ColorMap::Density }, DENSITY>
                            .ambiguous_with(ColorParticlesLabel),
                    )
                    .with_system(color_particles_by_quantity_system::<IonizedHydrogenFraction, { ColorMap::IonizedHydrogenFraction }, DIMENSIONLESS>
                            .ambiguous_with(ColorParticlesLabel),
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
    if let ColorMap::Temperature = visualization_parameters.color_map {
        let scale = get_scale(
            particles
                .iter()
                .map(|(_, internal_energy, mass)| get_temperature(**internal_energy / **mass)),
        );
        for (mut circle, internal_energy, mass) in particles.iter_mut() {
            circle.color = temperature_color_map(**internal_energy / **mass, scale);
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
        parameters.color_map = match key {
            KeyCode::I => ColorMap::IonizedHydrogenFraction,
            KeyCode::M => ColorMap::Mass,
            KeyCode::D => ColorMap::Density,
            KeyCode::L => ColorMap::TimestepLevel,
            KeyCode::T => ColorMap::Temperature,
            _ => parameters.color_map,
        }
    }
}

fn color_particles_by_quantity_system<C, const CO: ColorMap, const D: Dimension>(
    visualization_parameters: Res<VisualizationParameters>,
    mut particles: Particles<(&mut DrawCircle, &C)>,
) where
    C: Component + std::ops::Deref<Target = Quantity<Float, D>>,
{
    if CO == visualization_parameters.color_map {
        let scale: Quantity<Float, D> = get_scale(particles.iter().map(|(_, value)| **value));
        let map = CO.map();
        for (mut circle, value) in particles.iter_mut() {
            circle.color = map(value.value_unchecked() / scale.value_unchecked());
        }
    }
}
