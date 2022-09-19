use std::f64::consts::PI;

use bevy::prelude::*;

use super::LocalParticle;
use crate::density;
use crate::domain::ExchangeDataPlugin;
use crate::mass::Mass;
use crate::position::Position;
use crate::pressure;
use crate::units::Density;
use crate::units::Length;
use crate::units::Pressure;

pub struct HydrodynamicsPlugin;

#[derive(StageLabel)]
pub enum HydrodynamicsStages {
    Hydrodynamics,
}

impl Plugin for HydrodynamicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(HydrodynamicsStages::Hydrodynamics, compute_pressure_system)
            .add_plugin(ExchangeDataPlugin::<pressure::Pressure>::default())
            .add_plugin(ExchangeDataPlugin::<density::Density>::default());
    }
}

fn compute_pressure_system(
    mut pressures: Query<
        (
            &mut pressure::Pressure,
            &mut density::Density,
            &Position,
            &Mass,
        ),
        With<LocalParticle>,
    >,
    particles: Query<&Position, (With<pressure::Pressure>, With<Mass>, With<LocalParticle>)>,
) {
    let cutoff = Length::meters(100.0);
    let cutoff_squared = cutoff.squared();
    let poly_6 = 4.0 / (PI * cutoff.powi::<8>());
    let rest_density = Density::kilogram_per_square_meter(300.0);
    let gas_const = Pressure::pascals(2000.0) / rest_density;
    for (mut pressure, mut density, pos1, mass) in pressures.iter_mut() {
        **density = Density::zero();
        for pos2 in particles.iter() {
            {
                let distance_squared = pos1.distance_squared(pos2);

                if distance_squared < cutoff_squared {
                    **density += **mass * poly_6 * (cutoff_squared - distance_squared).powi::<3>();
                }
            }
            **pressure = gas_const * (**density - rest_density);
        }
    }
}
