use bevy::prelude::shape::Circle;
use bevy::prelude::*;

use crate::position::Position;
use crate::units::meter;

const CIRCLE_SIZE: f32 = 5.0;

pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_sprites_system)
            .add_system(position_to_translation_system);
    }
}

pub fn spawn_sprites_system(
    mut commands: Commands,
    cells: Query<(Entity, &Position)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, pos) in cells.iter() {
        let handle = meshes.add(Mesh::from(Circle::new(CIRCLE_SIZE)));
        let circle = ColorMesh2dBundle {
            mesh: handle.into(),
            transform: Transform::from_translation(position_to_translation(pos)),
            ..default()
        };
        commands.entity(entity).insert_bundle(circle);
    }
}

fn position_to_translation(position: &Position) -> Vec3 {
    let camera_zoom = meter(0.01);
    let x = position.0 / camera_zoom;
    let y = position.1 / camera_zoom;
    Vec3::new(*x.value() as f32, *y.value() as f32, 0.0)
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

pub fn position_to_translation_system(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = position_to_translation(position);
    }
}
