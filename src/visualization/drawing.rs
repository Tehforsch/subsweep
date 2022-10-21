use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;

use super::camera_transform::CameraTransform;
use super::VisualizationStage;
use super::CIRCLE_RADIUS;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::Length;
use crate::units::VecLength;

#[derive(AmbiguitySetLabel)]
pub struct DrawAmbiguitySet;

pub enum LengthOrPixels {
    Length(Length),
    Pixels(f64),
}

#[derive(Component)]
pub struct DrawCircle {
    pub position: VecLength,
    pub radius: LengthOrPixels,
    pub color: Color,
}

#[derive(Component)]
pub struct DrawRect {
    pub lower_left: VecLength,
    pub upper_right: VecLength,
    pub color: Color,
}

impl DrawCircle {
    pub fn from_position_and_color(position: VecLength, color: Color) -> Self {
        Self {
            position,
            color,
            radius: LengthOrPixels::Pixels(CIRCLE_RADIUS),
        }
    }
}
pub(super) trait IntoBundle {
    type Output: Bundle;
    fn get_bundle(&self, camera_transform: &CameraTransform) -> Self::Output;
    fn translation(&self) -> &VecLength;
    fn set_translation(&mut self, pos: &VecLength);
}

impl IntoBundle for DrawCircle {
    type Output = ShapeBundle;
    fn get_bundle(&self, camera_transform: &CameraTransform) -> Self::Output {
        let shape = shapes::Circle {
            radius: match self.radius {
                LengthOrPixels::Length(length) => camera_transform.length_to_pixels(length),
                LengthOrPixels::Pixels(pixels) => pixels as f32,
            },
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
    fn get_bundle(&self, camera_transform: &CameraTransform) -> Self::Output {
        let shape = shapes::Rectangle {
            extents: camera_transform.position_to_pixels(self.upper_right - self.lower_left),
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

#[derive(Named)]
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

impl<T: IntoBundle + Component + Sync + Send + 'static> RaxiomPlugin for DrawBundlePlugin<T> {
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(
            VisualizationStage::AddDrawComponents,
            spawn_visualization_item_system::<T>,
        )
        .add_system_to_stage(
            VisualizationStage::Draw,
            draw_translation_system::<T>.in_ambiguity_set(DrawAmbiguitySet),
        );
    }
}

fn spawn_visualization_item_system<T: Component + IntoBundle>(
    mut commands: Commands,
    query: Query<(Entity, &T), Without<Mesh2dHandle>>,
    transform: Res<CameraTransform>,
) {
    for (entity, item) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(item.get_bundle(&transform));
    }
}

pub(super) fn draw_translation_system<T: Component + IntoBundle>(
    mut query: Query<(&mut Transform, &T)>,
    camera_transform: Res<CameraTransform>,
) {
    for (mut transform, item) in query.iter_mut() {
        let pixel_pos = camera_transform.position_to_pixels(*item.translation());
        transform.translation.x = pixel_pos.x;
        transform.translation.y = pixel_pos.y;
    }
}
