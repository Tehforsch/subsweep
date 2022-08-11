use bevy::prelude::Component;

use crate::units::f32;

#[derive(Component, Debug)]
pub struct Velocity(pub f32::Velocity, pub f32::Velocity);
