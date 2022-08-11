use bevy::prelude::shape::Circle;
use bevy::prelude::*;

use crate::position::Position;
use crate::units::meter;

const CIRCLE_SIZE: f32 = 5.0;

pub fn spawn_sprites_system(
    mut commands: Commands,
    cells: Query<(Entity, &Position)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let camera_zoom = meter(0.01);
    for (entity, pos) in cells.iter() {
        let x = pos.0 / camera_zoom;
        let y = pos.1 / camera_zoom;
        let handle = meshes.add(Mesh::from(Circle::new(CIRCLE_SIZE)));
        let circle = ColorMesh2dBundle {
            mesh: handle.into(),
            transform: Transform::from_translation(Vec3::new(
                x.value() as f32,
                y.value() as f32,
                0.0,
            )),
            ..default()
        };
        commands.entity(entity).insert_bundle(circle);
    }
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
