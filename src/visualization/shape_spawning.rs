use bevy::prelude::shape::Circle;
use bevy::prelude::shape::RegularPolygon;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

use super::position_to_translation;
use super::CAMERA_ZOOM_METERS;
use crate::position::Position;
use crate::units::f32::meter;
use crate::units::f32::Length;
use crate::units::vec2;

#[derive(Component)]
pub(super) struct DrawCircle {
    pub position: vec2::Length,
    pub radius: Length,
    pub color: Color,
}

#[derive(Component)]
pub(super) struct DrawRect {
    pub lower_left: vec2::Length,
    pub upper_right: vec2::Length,
    pub color: Color,
}

pub(super) trait IntoMeshAndColor {
    fn color(&self) -> Color;
    fn mesh(&self) -> Mesh;
    fn transform(&self) -> Transform;
}

impl IntoMeshAndColor for DrawCircle {
    fn color(&self) -> Color {
        self.color
    }

    fn mesh(&self) -> Mesh {
        Mesh::from(Circle::new(
            *(self.radius / meter(CAMERA_ZOOM_METERS)).value(),
        ))
    }

    // Todo: make this more generic to allow panning etc. This will be used for the live update
    fn transform(&self) -> Transform {
        let translation = position_to_translation(&Position(self.position));
        Transform {
            translation,
            ..Default::default()
        }
    }
}

impl IntoMeshAndColor for DrawRect {
    fn color(&self) -> Color {
        self.color
    }

    fn mesh(&self) -> Mesh {
        const REDUCED_MARGIN: f32 = 0.05;
        Mesh::from(RegularPolygon::new(
            *((self.upper_right - self.lower_left).length()
                / meter(CAMERA_ZOOM_METERS * (1.0 + REDUCED_MARGIN)))
            .value()
                / 2.0,
            4,
        ))
    }

    fn transform(&self) -> Transform {
        let center = (self.lower_left + self.upper_right) * 0.5;
        let translation = position_to_translation(&Position(center));
        Transform {
            translation,
            rotation: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 4.0),
            ..Default::default()
        }
    }
}

pub(super) fn spawn_visualization_item_system<T: Component + IntoMeshAndColor>(
    mut commands: Commands,
    query: Query<(Entity, &T), Without<Mesh2dHandle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, item) in query.iter() {
        let handle = meshes.add(item.mesh());
        let material = color_materials.add(ColorMaterial {
            color: item.color(),
            ..default()
        });
        let mesh_bundle = ColorMesh2dBundle {
            mesh: handle.into(),
            material,
            transform: item.transform(),
            ..default()
        };
        commands.entity(entity).insert_bundle(mesh_bundle);
    }
}
