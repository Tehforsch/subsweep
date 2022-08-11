use bevy::prelude::Component;

#[derive(Component, Debug)]
pub struct Velocity(pub crate::units::Velocity, pub crate::units::Velocity);
