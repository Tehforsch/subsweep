pub mod remote;
mod shape_spawning;

use bevy::prelude::*;

use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use self::shape_spawning::spawn_visualization_item_system;
use self::shape_spawning::DrawCircle;
use self::shape_spawning::DrawRect;
use crate::communication::Rank;
use crate::domain::quadtree::QuadTree;
use crate::physics::LocalParticle;
use crate::physics::PhysicsStages;
use crate::physics::RemoteParticle;
use crate::position::Position;
use crate::units::f32::meter;
use crate::units::vec2::Length;

const COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW];

pub const CAMERA_ZOOM_METERS: f32 = 0.01;

#[derive(StageLabel)]
pub enum VisualizationStage {
    Synchronize,
    AddVisualization,
    Draw,
}

pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        let rank = *app.world.get_resource::<Rank>().unwrap();
        app.add_stage_after(
            PhysicsStages::Gravity,
            VisualizationStage::Synchronize,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            VisualizationStage::Synchronize,
            VisualizationStage::AddVisualization,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            VisualizationStage::AddVisualization,
            VisualizationStage::Draw,
            SystemStage::parallel(),
        );
        if rank == 0 {
            app.add_startup_system(setup_camera_system)
                .add_system_to_stage(
                    VisualizationStage::Synchronize,
                    receive_particles_on_main_thread_system,
                )
                .add_system_to_stage(VisualizationStage::AddVisualization, spawn_sprites_system)
                .add_system_to_stage(
                    VisualizationStage::Draw,
                    spawn_visualization_item_system::<DrawCircle>,
                )
                .add_system_to_stage(
                    VisualizationStage::Draw,
                    spawn_visualization_item_system::<DrawRect>,
                )
                .add_system_to_stage(
                    VisualizationStage::AddVisualization,
                    position_to_translation_system,
                )
                .add_system_to_stage(VisualizationStage::AddVisualization, show_quadtree_system);
        } else {
            app.add_system_to_stage(
                VisualizationStage::Synchronize,
                send_particles_to_main_thread_system,
            );
        }
    }
}

fn spawn_sprites_system(
    mut commands: Commands,
    local_cells: Query<
        (Entity, &Position),
        (
            With<LocalParticle>,
            Without<RemoteParticle>,
            Without<DrawCircle>,
        ),
    >,
    remote_cells: Query<
        (Entity, &Position, &RemoteParticle),
        (Without<LocalParticle>, Without<DrawCircle>),
    >,
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
        let color = COLORS[rank as usize];
        commands.entity(entity).insert(DrawCircle {
            position: pos.0,
            radius: meter(0.05),
            color,
        });
    }
}

#[derive(Component)]
struct Outline;

fn show_quadtree_system(
    mut commands: Commands,
    quadtree: Res<QuadTree>,
    outlines: Query<Entity, With<Outline>>,
) {
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    quadtree.depth_first_map(&mut |extents| {
        let lower_left = Length::new(extents.x_min, extents.y_min);
        let upper_right = Length::new(extents.x_max, extents.y_max);
        commands.spawn().insert(Outline).insert(DrawRect {
            lower_left,
            upper_right,
            color: Color::GREEN,
        });
    });
}

fn position_to_translation(position: &Position) -> Vec3 {
    let camera_zoom = meter(CAMERA_ZOOM_METERS);
    let pos = *(position.0 / camera_zoom).value();
    Vec3::new(pos.x, pos.y, 0.0)
}

pub fn position_to_translation_system(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = position_to_translation(position);
    }
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
