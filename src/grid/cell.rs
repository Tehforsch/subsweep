use bevy::prelude::Component;
use bevy::prelude::Entity;

use crate::communication::Identified;

pub struct Neighbour {
    pub entity: Entity,
    pub kind: NeighbourKind,
}

pub enum NeighbourKind {
    Local,
    Remote(Identified<Entity>),
}

#[derive(Component)]
pub struct Cell {
    pub neighbours: Vec<Neighbour>,
}
