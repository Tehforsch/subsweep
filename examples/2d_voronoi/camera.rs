use bevy::prelude::*;

const SCALE: f32 = 900.0;

#[derive(Component)]
pub struct WorldCamera;

#[derive(Debug, Default, Clone, Resource)]
pub struct MousePosition(pub Vec2);

pub fn setup_camera_system(mut commands: Commands) {
    let camera = Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(1.0 / SCALE, 1.0 / SCALE, 1.0)),
        ..default()
    };
    commands.spawn((WorldCamera, camera));
    commands.insert_resource(MousePosition(Vec2::new(0.0, 0.0)));
}

pub fn track_mouse_world_position_system(
    windows: Res<Windows>,
    mut position: ResMut<MousePosition>,
    camera_query: Query<(&Camera, &Transform), With<WorldCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = windows.get_primary().unwrap();
    let cursor_pos_window = window.cursor_position();
    if let Some(cursor_pos_window) = cursor_pos_window {
        let size = camera.logical_viewport_size().unwrap();
        let p = cursor_pos_window - size / 2.0;
        let world_pos = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        position.0.x = world_pos.x;
        position.0.y = world_pos.y;
    }
}
