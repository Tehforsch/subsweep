use bevy::prelude::Component;

use crate::units::vec2::Length;

#[derive(Component, Debug)]
pub struct Position(pub Length);
