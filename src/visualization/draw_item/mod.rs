use std::marker::PhantomData;

pub(super) mod circle;
pub(super) mod rect;

use bevy::ecs::system::AsSystemLabel;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;

use super::camera_transform::CameraTransform;
use super::remote::receive_items_on_main_thread_system;
use super::remote::send_items_to_main_thread_system;
use super::VisualizationStage;
use crate::communication::CommunicationPlugin;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::VecLength;

#[derive(Equivalence, Deref, DerefMut, Clone, Debug)]
pub struct Pixels(f64);

#[derive(Named)]
pub struct DrawItemSyncOrdering;

#[derive(AmbiguitySetLabel)]
pub struct DrawAmbiguitySet;

pub(super) trait DrawItem {
    type Output: Bundle;
    fn get_bundle(&self, camera_transform: &CameraTransform) -> Self::Output;
    fn translation(&self) -> &VecLength;
    fn set_translation(&mut self, pos: &VecLength);
}

#[derive(Named)]
pub(super) struct DrawItemPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for DrawItemPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Equivalence + DrawItem + Component + Clone + Sync + Send + 'static> RaxiomPlugin
    for DrawItemPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(CommunicationPlugin::<T>::sync());
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_well_ordered_system_to_stage::<_, DrawItemSyncOrdering>(
            VisualizationStage::Synchronize,
            receive_items_on_main_thread_system::<T>,
            receive_items_on_main_thread_system::<T>.as_system_label(),
        )
        .add_system_to_stage(
            VisualizationStage::AddDrawComponents,
            insert_meshes_system::<T>,
        )
        .add_system_to_stage(
            VisualizationStage::AddDrawComponents,
            insert_meshes_system::<T>,
        )
        .add_system_to_stage(
            VisualizationStage::Draw,
            draw_translation_system::<T>.in_ambiguity_set(DrawAmbiguitySet),
        );
    }

    fn build_on_other_ranks(&self, sim: &mut Simulation) {
        sim.add_well_ordered_system_to_stage::<_, DrawItemSyncOrdering>(
            VisualizationStage::Synchronize,
            send_items_to_main_thread_system::<T>,
            send_items_to_main_thread_system::<T>.as_system_label(),
        );
    }
}

fn insert_meshes_system<T: Component + DrawItem>(
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

pub(super) fn draw_translation_system<T: Component + DrawItem>(
    mut query: Query<(&mut Transform, &T)>,
    camera_transform: Res<CameraTransform>,
) {
    for (mut transform, item) in query.iter_mut() {
        let pixel_pos = camera_transform.position_to_pixels(*item.translation());
        transform.translation.x = pixel_pos.x;
        transform.translation.y = pixel_pos.y;
    }
}
