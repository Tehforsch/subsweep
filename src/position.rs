use bevy::prelude::Component;

use crate::units::f64::Length;

#[derive(Component, Debug)]
pub struct Position(pub Length, pub Length);
