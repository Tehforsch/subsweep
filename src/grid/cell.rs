use bevy::prelude::Component;
use bevy::prelude::Entity;

use crate::communication::Identified;
use crate::units::VecDimensionless;

#[cfg(feature = "2d")]
pub type FaceArea = crate::units::Length;

#[cfg(not(feature = "2d"))]
pub type FaceArea = crate::units::Area;

#[derive(Clone)]
pub enum Neighbour {
    Local(Entity),
    Remote(RemoteNeighbour),
}

impl Neighbour {
    pub fn local_entity(&self) -> Entity {
        match self {
            Neighbour::Local(entity) => *entity,
            Neighbour::Remote(remote) => remote.local_entity,
        }
    }
}

#[derive(Clone)]
pub struct RemoteNeighbour {
    pub local_entity: Entity,
    pub remote_entity: Identified<Entity>,
}

#[derive(Component)]
pub struct Cell {
    pub neighbours: Vec<(Face, Neighbour)>,
}

#[derive(Clone)]
pub struct Face {
    pub area: FaceArea,
    pub normal: VecDimensionless,
}
