use bevy::prelude::*;

use super::CameraTransform;
use crate::domain::GlobalExtent;

#[derive(Component)]
pub(super) struct WorldCamera;

pub(super) fn setup_camera_system(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(WorldCamera);
}

pub(super) fn camera_translation_system(
    mut camera: Query<&mut Transform, With<WorldCamera>>,
    extent: Res<GlobalExtent>,
    camera_transform: Res<CameraTransform>,
) {
    let mut camera = camera.single_mut();
    let pos = camera_transform.position_to_pixels(extent.center);
    camera.translation.x = pos.x;
    camera.translation.y = pos.y;
}

pub(super) fn camera_scale_system(
    extent: Res<GlobalExtent>,
    mut camera_transform: ResMut<CameraTransform>,
    windows: Res<Windows>,
) {
    let length = extent.max_side_length();
    let window = windows.primary();
    let max_side = window.width().max(window.height()).min(1000.0);
    *camera_transform = CameraTransform::from_scale(length / (max_side as f64));
}
