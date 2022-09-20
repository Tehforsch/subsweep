use std::f64::consts::PI;

use bevy::prelude::*;

use super::LocalParticle;
use super::Timestep;
use crate::density;
use crate::domain::ExchangeDataPlugin;
use crate::mass;
use crate::mass::Mass;
use crate::named::Named;
use crate::plugin_utils::Simulation;
use crate::plugin_utils::TenetPlugin;
use crate::position::Position;
use crate::pressure;
use crate::units::Density;
use crate::units::Length;
use crate::units::Pressure;
use crate::units::VecAcceleration;
use crate::velocity::Velocity;

pub struct HydrodynamicsPlugin;

#[derive(StageLabel)]
pub enum HydrodynamicsStages {
    Hydrodynamics,
}

const CUTOFF_LENGTH: Length = Length::astronomical_units(0.1);

impl Named for HydrodynamicsPlugin {
    fn name() -> &'static str {
        "hydrodynamics"
    }
}

impl TenetPlugin for HydrodynamicsPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(
            HydrodynamicsStages::Hydrodynamics,
            compute_pressure_and_density_system,
        )
        .add_system_to_stage(
            HydrodynamicsStages::Hydrodynamics,
            compute_forces_system.after(compute_pressure_and_density_system),
        )
        .add_startup_system_to_stage(
            StartupStage::PostStartup,
            insert_pressure_and_density_system,
        )
        .add_plugin(ExchangeDataPlugin::<pressure::Pressure>::default())
        .add_plugin(ExchangeDataPlugin::<density::Density>::default());
    }
}

fn insert_pressure_and_density_system(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<LocalParticle>,
            Without<pressure::Pressure>,
            Without<density::Density>,
        ),
    >,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert_bundle((pressure::Pressure::default(), density::Density::default()));
    }
}
fn compute_pressure_and_density_system(
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
    let cutoff_squared = CUTOFF_LENGTH.squared();
    let poly_6 = 4.0 / (PI * CUTOFF_LENGTH.powi::<8>());
    let rest_density = Density::kilogram_per_square_meter(300.0);
    let gas_const = Pressure::pascals(100000.0) / rest_density;
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

fn compute_forces_system(
    mut particles1: Query<(
        Entity,
        &mut Velocity,
        &Position,
        &pressure::Pressure,
        &density::Density,
    )>,
    particles2: Query<(
        Entity,
        &Position,
        &pressure::Pressure,
        &density::Density,
        &mass::Mass,
    )>,
    timestep: Res<Timestep>,
) {
    let spiky_grad = -10.0 / (PI * CUTOFF_LENGTH.powi::<5>());
    for (entity1, mut vel, pos1, pressure1, density1) in particles1.iter_mut() {
        let mut acc = VecAcceleration::zero();
        for (entity2, pos2, pressure2, density2, mass2) in particles2.iter() {
            if entity1 == entity2 {
                continue;
            }

            let distance = **pos2 - **pos1;
            let distance_normalized = distance.normalize();
            let length = distance.length();

            if length < CUTOFF_LENGTH {
                acc += distance_normalized * **mass2 * (**pressure1 + **pressure2)
                    / (2.0 * **density2)
                    * spiky_grad
                    * (CUTOFF_LENGTH - length).cubed()
                    / **density1;
            }
        }
        **vel += acc * **timestep;
    }
}
