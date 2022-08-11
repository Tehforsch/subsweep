use bevy::prelude::Component;

use crate::units::vec2;

#[derive(Component, Debug)]
pub struct Velocity(pub vec2::Velocity);
