pub mod remote;

use bevy::prelude::shape::Circle;
use bevy::prelude::shape::RegularPolygon;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use crate::communication::Rank;
use crate::domain::QuadTree;
use crate::physics::LocalParticle;
use crate::physics::RemoteParticle;
use crate::position::Position;
use crate::units::f32::meter;
use crate::units::vec2;
use crate::units::vec2::Length;
const CIRCLE_SIZE: f32 = 5.0;

const COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW];

#[derive(StageLabel)]
pub enum VisualizationStage {
    Synchronize,
    Visualize,
}

pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        let rank = *app.world.get_resource::<Rank>().unwrap();
        app.add_stage_after(
            CoreStage::Update,
            VisualizationStage::Synchronize,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            VisualizationStage::Synchronize,
            VisualizationStage::Visualize,
            SystemStage::parallel(),
        );
        if rank == 0 {
            app.add_startup_system(setup_camera_system)
                .add_system_to_stage(
                    VisualizationStage::Synchronize,
                    receive_particles_on_main_thread_system,
                )
                .add_system_to_stage(VisualizationStage::Visualize, spawn_sprites_system)
                .add_system_to_stage(
                    VisualizationStage::Visualize,
                    position_to_translation_system,
                )
                .add_system_to_stage(VisualizationStage::Visualize, show_quadtree_system);
        } else {
            app.add_system_to_stage(
                VisualizationStage::Synchronize,
                send_particles_to_main_thread_system,
            );
        }
    }
}

pub fn spawn_sprites_system(
    mut commands: Commands,
    local_cells: Query<
        (Entity, &Position),
        (
            With<LocalParticle>,
            Without<RemoteParticle>,
            Without<Mesh2dHandle>,
        ),
    >,
    remote_cells: Query<
        (Entity, &Position, &RemoteParticle),
        (Without<LocalParticle>, Without<Mesh2dHandle>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, pos, rank) in local_cells
        .iter()
        .map(|(entity, pos)| (entity, pos, 0))
        .chain(
            remote_cells
                .iter()
                .map(|(entity, pos, rank)| (entity, pos, rank.0)),
        )
    {
        let handle = meshes.add(Mesh::from(Circle::new(CIRCLE_SIZE)));
        let color = COLORS[rank as usize];
        let material = color_materials.add(ColorMaterial { color, ..default() });
        let circle = ColorMesh2dBundle {
            mesh: handle.into(),
            material,
            transform: Transform::from_translation(position_to_translation(pos)),
            ..default()
        };
        commands.entity(entity).insert_bundle(circle);
    }
}

fn position_to_translation(position: &Position) -> Vec3 {
    let camera_zoom = meter(0.01);
    let pos = *(position.0 / camera_zoom).value();
    Vec3::new(pos.x, pos.y, 0.0)
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

pub fn position_to_translation_system(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = position_to_translation(position);
    }
}

#[derive(Component)]
struct Outline;

fn show_quadtree_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    particles: Query<(Entity, &Position)>,
    outlines: Query<&Outline>,
) {
    if outlines.iter().next().is_some() {
        return;
    }
    let particles: Vec<(vec2::Length, Entity)> = particles
        .iter()
        .map(|(entity, pos)| (pos.0, entity))
        .collect();
    let quadtree = QuadTree::new(particles);
    quadtree.depth_first_map(&mut |extents| {
        let center = Length::new(extents.x_center, extents.y_center);
        let handle = meshes.add(Mesh::from(RegularPolygon::new(
            67.0 * (extents.x_max.unwrap_value() - extents.x_min.unwrap_value()),
            4,
        )));
        let circle = ColorMesh2dBundle {
            mesh: handle.into(),
            transform: Transform {
                translation: position_to_translation(&Position(center)),
                rotation: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 4.0),
                ..default()
            },
            ..default()
        };
        commands.spawn().insert(Outline).insert_bundle(circle);
    });
}
