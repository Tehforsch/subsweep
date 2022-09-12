use bevy::prelude::Plugin;

use super::gravity_system;
use crate::physics::PhysicsStages;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(PhysicsStages::Gravity, gravity_system);
    }
}
