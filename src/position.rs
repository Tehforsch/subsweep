use bevy::prelude::Component;

use crate::units::Length;

#[derive(Component, Debug)]
pub struct Position(pub Length, pub Length);
