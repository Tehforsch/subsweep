mod drawing;
mod parameters;
pub mod remote;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
pub use drawing::DrawCircle;
pub use drawing::DrawRect;
use lazy_static::lazy_static;
use mpi::Rank;

use self::drawing::draw_translation_system;
use self::drawing::DrawBundlePlugin;
use self::drawing::IntoBundle;
use self::parameters::Parameters;
use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use self::remote::ParticleVisualizationExchangeData;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::domain::show_segment_extent_system;
use crate::parameters::ParameterPlugin;
use crate::physics::LocalParticle;
use crate::physics::LocalQuadTree;
use crate::physics::RemoteParticle;
use crate::physics::RemoteQuadTree;
use crate::plugin_utils::is_main_rank;
use crate::position::Position;
use crate::units::Length;

const COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW];

lazy_static! {
    pub static ref CAMERA_ZOOM: Length = Length::meter(0.01);
}

#[derive(StageLabel)]
pub enum VisualizationStage {
    Synchronize,
    AddVisualization,
    AddDrawComponents,
    Draw,
}

pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::PostUpdate,
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
            VisualizationStage::AddDrawComponents,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            VisualizationStage::AddDrawComponents,
            VisualizationStage::Draw,
            SystemStage::parallel(),
        );
        if is_main_rank(app) {
            app.add_plugin(ParameterPlugin::<Parameters>::new("visualization"))
                .add_plugin(DrawBundlePlugin::<DrawRect>::default())
                .add_plugin(DrawBundlePlugin::<DrawCircle>::default())
                .add_plugin(ShapePlugin)
                .add_plugin(
                    CommunicationPlugin::<ParticleVisualizationExchangeData>::new(
                        CommunicationType::Sync,
                    ),
                )
                .add_startup_system(setup_camera_system)
                .add_system_to_stage(
                    VisualizationStage::Synchronize,
                    receive_particles_on_main_thread_system,
                )
                .add_system_to_stage(VisualizationStage::AddVisualization, spawn_sprites_system)
                .add_system_to_stage(
                    VisualizationStage::Draw,
                    position_to_translation_system::<DrawCircle>
                        .before(draw_translation_system::<DrawCircle>),
                );
            if app
                .world
                .get_resource::<Parameters>()
                .unwrap()
                .show_remote_quadtree
            {
                app.add_system_to_stage(
                    VisualizationStage::AddVisualization,
                    show_remote_quadtree_system,
                );
            }
            if app
                .world
                .get_resource::<Parameters>()
                .unwrap()
                .show_local_quadtree
            {
                app.add_system_to_stage(
                    VisualizationStage::AddVisualization,
                    show_local_quadtree_system,
                );
            }
            if app
                .world
                .get_resource::<Parameters>()
                .unwrap()
                .show_segment_extent
            {
                app.add_system_to_stage(
                    VisualizationStage::AddVisualization,
                    show_segment_extent_system,
                );
            }
        } else {
            app.add_plugin(
                CommunicationPlugin::<ParticleVisualizationExchangeData>::new(
                    CommunicationType::Sync,
                ),
            )
            .add_system_to_stage(
                VisualizationStage::Synchronize,
                send_particles_to_main_thread_system,
            );
        }
    }
}

pub fn get_color(rank: Rank) -> Color {
    COLORS[(rank as usize).rem_euclid(COLORS.len())]
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
        let color = get_color(rank);
        commands.entity(entity).insert(DrawCircle {
            position: pos.0,
            radius: Length::meter(0.05),
            color,
        });
    }
}

#[derive(Component)]
struct Outline;
#[derive(Component)]
struct RemoteOutline;

fn show_remote_quadtree_system(
    mut commands: Commands,
    quadtree: Res<RemoteQuadTree>,
    outlines: Query<Entity, With<RemoteOutline>>,
) {
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    quadtree.depth_first_map(&mut |extents, l| {
        let color = {
            if l.len() == 1 {
                Some(get_color(l.first().unwrap().1.rank))
            } else if l.len() > 1 {
                Some(Color::WHITE)
            } else {
                None
            }
        };
        if let Some(color) = color {
            commands.spawn().insert(RemoteOutline).insert(DrawRect {
                lower_left: extents.min,
                upper_right: extents.max,
                color,
            });
        }
    });
}

fn show_local_quadtree_system(
    mut commands: Commands,
    quadtree: Res<LocalQuadTree>,
    outlines: Query<Entity, With<Outline>>,
) {
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    quadtree.depth_first_map(&mut |extents, _| {
        commands.spawn().insert(Outline).insert(DrawRect {
            lower_left: extents.min,
            upper_right: extents.max,
            color: get_color(0),
        });
    });
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn position_to_translation_system<T: Component + IntoBundle>(
    mut query: Query<(&mut T, &Position)>,
) {
    for (mut item, position) in query.iter_mut() {
        item.set_translation(&position.0);
    }
}
