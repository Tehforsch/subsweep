use bevy::prelude::Component;

use crate::units::f64;

#[derive(Component, Debug)]
pub struct Velocity(pub f64::Velocity, pub f64::Velocity);
