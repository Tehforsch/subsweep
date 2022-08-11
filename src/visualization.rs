use bevy::prelude::*;

use crate::position::Position;
use crate::units::meter;
use crate::units::Length;

const GRID_SIZE: f32 = 20.0;

pub fn spawn_sprites_system(mut commands: Commands, cells: Query<(Entity, &Position)>) {
    for (entity, pos) in cells.iter() {
        commands
            .entity(entity)
            .insert_bundle(get_sprite_at_position(pos.0, pos.1));
    }
}

fn get_sprite_at_position(x: Length, y: Length) -> SpriteBundle {
    let x = x / meter(1.0);
    let y = y / meter(1.0);
    SpriteBundle {
        transform: Transform {
            translation: Vec3::new(
                GRID_SIZE * x.value() as f32 - 400.0,
                GRID_SIZE * y.value() as f32 - 400.0,
                0.0,
            ),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(GRID_SIZE, GRID_SIZE)),
            ..default()
        },
        ..default()
    }
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
