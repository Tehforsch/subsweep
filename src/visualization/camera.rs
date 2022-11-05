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
    let pos = camera_transform.position_to_pixels(extent.center());
    camera.translation.x = pos.x;
    camera.translation.y = pos.y;
}

pub(super) fn camera_scale_system(
    extent: Res<GlobalExtent>,
    mut camera_transform: ResMut<CameraTransform>,
    windows: Res<Windows>,
) {
    let simulation_width = extent.side_lengths().x();
    let simulation_height = extent.side_lengths().y();
    let window = windows.primary();
    let window_width = window.width().max(1000.0);
    let window_height = window.height().max(1000.0);
    let max_ratio =
        (simulation_width / window_width as f64).max(simulation_height / window_height as f64);
    *camera_transform = CameraTransform::from_scale(0.5 * max_ratio);
}
