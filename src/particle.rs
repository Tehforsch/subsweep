use bevy::prelude::Bundle;

use crate::mass::Mass;
use crate::physics::LocalParticle;
use crate::position::Position;
use crate::velocity::Velocity;

#[derive(Bundle)]
pub struct LocalParticleBundle {
    pos: Position,
    vel: Velocity,
    mass: Mass,
    _local: LocalParticle,
}

impl LocalParticleBundle {
    pub fn new(pos: Position, vel: Velocity, mass: Mass) -> Self {
        Self {
            pos,
            vel,
            mass,
            _local: LocalParticle,
        }
    }
}
