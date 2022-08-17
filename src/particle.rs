use bevy::prelude::Bundle;

use crate::communication::Rank;
use crate::physics::LocalParticle;
use crate::physics::RemoteParticle;
use crate::position::Position;
use crate::velocity::Velocity;

#[derive(Bundle)]
pub struct LocalParticleBundle {
    pos: Position,
    vel: Velocity,
    _local: LocalParticle,
}

impl LocalParticleBundle {
    pub fn new(pos: Position, vel: Velocity) -> Self {
        Self {
            pos,
            vel,
            _local: LocalParticle,
        }
    }
}

#[derive(Bundle)]
pub struct RemoteParticleBundle {
    pos: Position,
    remote: RemoteParticle,
}

impl RemoteParticleBundle {
    pub fn new(pos: Position, rank: Rank) -> Self {
        Self {
            pos,
            remote: RemoteParticle(rank),
        }
    }
}
