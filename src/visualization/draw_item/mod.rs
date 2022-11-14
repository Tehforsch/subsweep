pub(super) mod circle;
pub(super) mod rect;

use std::marker::PhantomData;

use bevy::ecs::system::AsSystemLabel;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy::utils::HashMap;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;

use super::camera_transform::CameraTransform;
use super::remote::receive_items_on_main_thread_system;
use super::remote::send_items_to_main_thread_system;
use super::RColor;
use super::VisualizationStage;
use crate::communication::CommunicationPlugin;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::VecLength;

#[derive(Equivalence, Deref, DerefMut, Clone, Debug)]
pub struct Pixels(pub f64);

#[derive(Named)]
pub struct DrawItemSyncOrdering;

pub trait DrawItem {
    fn get_color(&self) -> RColor;
    fn translation(&self) -> &VecLength;
    fn set_translation(&mut self, pos: &VecLength);
    fn get_mesh() -> Mesh;
    fn get_scale(&self, camera_transform: &super::CameraTransform) -> Vec2;
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

#[derive(Resource)]
pub(super) struct MeshHandle<T> {
    _marker: PhantomData<T>,
    handle: Handle<Mesh>,
}

#[derive(Default, Resource)]
pub(super) struct MaterialsCache {
    map: HashMap<RColor, Handle<ColorMaterial>>,
}

impl MaterialsCache {
    fn get_material(
        &mut self,
        color: RColor,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        let material = ColorMaterial {
            color: color.into(),
            ..default()
        };
        if self.map.contains_key(&color) {
            self.map[&color].clone()
        } else {
            let handle = materials.add(material);
            self.map.insert(color, handle.clone());
            handle
        }
    }
}

#[derive(SystemLabel)]
struct DrawTranslationLabel;

#[derive(SystemLabel)]
struct ChangeColorsLabel;

#[derive(SystemLabel)]
struct InsertMeshesLabel;

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

    fn build_once_on_main_rank(&self, sim: &mut Simulation) {
        sim.insert_resource(MaterialsCache::default());
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_startup_system(setup_meshes_system::<T>.ignore_all_ambiguities())
            .add_well_ordered_system_to_stage::<_, DrawItemSyncOrdering>(
                VisualizationStage::Synchronize,
                receive_items_on_main_thread_system::<T>,
                receive_items_on_main_thread_system::<T>.as_system_label(),
            )
            .add_system_to_stage(
                VisualizationStage::AddDrawComponents,
                insert_meshes_system::<T>
                    .label(InsertMeshesLabel)
                    .ambiguous_with(InsertMeshesLabel),
            )
            .add_system_to_stage(
                VisualizationStage::Draw,
                change_colors_system::<T>
                    .label(ChangeColorsLabel)
                    .ambiguous_with(ChangeColorsLabel),
            )
            .add_system_to_stage(
                VisualizationStage::Draw,
                draw_translation_system::<T>
                    .label(DrawTranslationLabel)
                    .ambiguous_with(DrawTranslationLabel),
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

fn setup_meshes_system<T: DrawItem + Send + Sync + 'static>(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let handle = meshes.add(T::get_mesh());
    commands.insert_resource(MeshHandle {
        handle,
        _marker: PhantomData::<T>,
    });
}

pub(super) fn insert_meshes_system<T: Component + DrawItem>(
    mut cache: ResMut<MaterialsCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    query: Query<(&T, Entity), Without<Mesh2dHandle>>,
    mesh_handle: Res<MeshHandle<T>>,
) {
    for (item, entity) in query.iter() {
        commands.entity(entity).insert(ColorMesh2dBundle {
            mesh: Mesh2dHandle(mesh_handle.handle.clone()),
            material: cache.get_material(item.get_color(), &mut materials),
            ..default()
        });
    }
}

pub(super) fn change_colors_system<T: Component + DrawItem>(
    mut query: Query<(&mut Handle<ColorMaterial>, &T)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cache: ResMut<MaterialsCache>,
) {
    for (mut handle, item) in query.iter_mut() {
        *handle = cache.get_material(item.get_color(), &mut materials);
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
        let scale = item.get_scale(&camera_transform);
        transform.scale.x = scale.x;
        transform.scale.y = scale.y;
    }
}
