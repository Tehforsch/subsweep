use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;

use super::CAMERA_ZOOM;
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

pub(super) trait IntoBundle {
    type Output: Bundle;
    fn into_bundle(&self) -> Self::Output;
}

impl IntoBundle for DrawCircle {
    type Output = ShapeBundle;
    fn into_bundle(&self) -> Self::Output {
        let shape = shapes::Circle {
            radius: self.radius.in_units(*CAMERA_ZOOM),
            center: self.position.in_units(*CAMERA_ZOOM),
        };

        GeometryBuilder::build_as(
            &shape,
            DrawMode::Fill(FillMode::color(self.color)),
            Transform::default(),
        )
    }
}

impl IntoBundle for DrawRect {
    type Output = ShapeBundle;
    fn into_bundle(&self) -> Self::Output {
        let center = (self.upper_right + self.lower_left).in_units(*CAMERA_ZOOM) * 0.5;
        let shape = shapes::Rectangle {
            extents: (self.upper_right - self.lower_left).in_units(*CAMERA_ZOOM),
            origin: RectangleOrigin::CustomCenter(center),
        };

        GeometryBuilder::build_as(
            &shape,
            DrawMode::Stroke(StrokeMode::new(self.color, 2.0)),
            Transform::default(),
        )
    }
}

pub(super) fn spawn_visualization_item_system<T: Component + IntoBundle>(
    mut commands: Commands,
    query: Query<(Entity, &T), Without<Mesh2dHandle>>,
) {
    for (entity, item) in query.iter() {
        commands.entity(entity).insert_bundle(item.into_bundle());
    }
}
