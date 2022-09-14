use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;

use super::VisualizationStage;
use super::CAMERA_ZOOM;
use crate::units::Length;
use crate::units::VecLength;

#[derive(Component)]
pub struct DrawCircle {
    pub position: VecLength,
    pub radius: Length,
    pub color: Color,
}

#[derive(Component)]
pub struct DrawRect {
    pub lower_left: VecLength,
    pub upper_right: VecLength,
    pub color: Color,
}

pub(super) trait IntoBundle {
    type Output: Bundle;
    fn into_bundle(&self) -> Self::Output;
    fn translation(&self) -> &VecLength;
    fn set_translation(&mut self, pos: &VecLength);
}

impl IntoBundle for DrawCircle {
    type Output = ShapeBundle;
    fn into_bundle(&self) -> Self::Output {
        let shape = shapes::Circle {
            radius: self.radius.in_units(CAMERA_ZOOM),
            center: Vec2::new(0.0, 0.0),
        };

        GeometryBuilder::build_as(
            &shape,
            DrawMode::Fill(FillMode::color(self.color)),
            Transform::default(),
        )
    }

    fn translation(&self) -> &VecLength {
        &self.position
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.position = *pos;
    }
}

impl IntoBundle for DrawRect {
    type Output = ShapeBundle;
    fn into_bundle(&self) -> Self::Output {
        let shape = shapes::Rectangle {
            extents: (self.upper_right - self.lower_left).in_units(CAMERA_ZOOM),
            origin: RectangleOrigin::BottomLeft,
        };

        GeometryBuilder::build_as(
            &shape,
            DrawMode::Stroke(StrokeMode::new(self.color, 2.0)),
            Transform::default(),
        )
    }

    fn translation(&self) -> &VecLength {
        &self.lower_left
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.lower_left = *pos;
    }
}

pub(super) struct DrawBundlePlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for DrawBundlePlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: IntoBundle + Component + Sync + Send + 'static> Plugin for DrawBundlePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            VisualizationStage::AddDrawComponents,
            spawn_visualization_item_system::<T>,
        )
        .add_system_to_stage(VisualizationStage::Draw, draw_translation_system::<T>);
    }
}

fn spawn_visualization_item_system<T: Component + IntoBundle>(
    mut commands: Commands,
    query: Query<(Entity, &T), Without<Mesh2dHandle>>,
) {
    for (entity, item) in query.iter() {
        commands.entity(entity).insert_bundle(item.into_bundle());
    }
}

fn position_to_translation(position: &VecLength) -> Vec3 {
    let pos = position.in_units(CAMERA_ZOOM);
    Vec3::new(pos.x, pos.y, 0.0)
}

pub(super) fn draw_translation_system<T: Component + IntoBundle>(
    mut query: Query<(&mut Transform, &T)>,
) {
    for (mut transform, item) in query.iter_mut() {
        transform.translation = position_to_translation(item.translation());
    }
}
