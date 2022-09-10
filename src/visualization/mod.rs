mod drawing;
mod parameters;
pub mod remote;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
pub use drawing::DrawCircle;
pub use drawing::DrawRect;
use lazy_static::lazy_static;

use self::drawing::draw_translation_system;
use self::drawing::DrawBundlePlugin;
use self::drawing::IntoBundle;
use self::parameters::Parameters;
use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use self::remote::ParticleVisualizationExchangeData;
use self::remote::RemoteParticleVisualization;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Rank;
use crate::communication::WorldRank;
use crate::domain::quadtree::QuadTree;
use crate::parameters::ParameterPlugin;
use crate::physics::LocalParticle;
use crate::physics::PhysicsStages;
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
        let rank = *app.world.get_resource::<WorldRank>().unwrap();
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
            VisualizationStage::AddDrawComponents,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            VisualizationStage::AddDrawComponents,
            VisualizationStage::Draw,
            SystemStage::parallel(),
        );
        if rank.is_main() {
            app.add_plugin(ParameterPlugin::<Parameters>::new("visualization"))
                .add_plugin(ShapePlugin)
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
                .add_system_to_stage(
                    VisualizationStage::AddVisualization,
                    spawn_sprites_system::<LocalParticle>,
                )
                .add_system_to_stage(
                    VisualizationStage::AddVisualization,
                    spawn_sprites_system::<RemoteParticleVisualization>,
                )
                .add_system_to_stage(
                    VisualizationStage::Draw,
                    position_to_translation_system::<DrawCircle>
                        .before(draw_translation_system::<DrawCircle>),
                );
            if app
                .world
                .get_resource::<Parameters>()
                .unwrap()
                .show_quadtree
            {
                app.add_system_to_stage(VisualizationStage::AddVisualization, show_quadtree_system);
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

fn spawn_sprites_system<T: Component + GetColor>(
    mut commands: Commands,
    particles: Query<(Entity, &Position, &T), (With<T>, Without<DrawCircle>)>,
) {
    for (entity, pos, colored) in particles.iter() {
        commands.entity(entity).insert(DrawCircle {
            position: **pos,
            radius: Length::meter(0.05),
            color: colored.get_color(),
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
    todo!()
    // quadtree.depth_first_map(&mut |extent, _| {
    //     commands.spawn().insert(Outline).insert(DrawRect {
    //         lower_left: extent.min,
    //         upper_right: extent.max,
    //         color: Color::GREEN,
    //     });
    // });
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn position_to_translation_system<T: Component + IntoBundle>(
    mut query: Query<(&mut T, &Position)>,
) {
    for (mut item, position) in query.iter_mut() {
        item.set_translation(position);
    }
}

trait GetColor {
    fn get_color(&self) -> Color;
}

impl GetColor for LocalParticle {
    fn get_color(&self) -> Color {
        get_color(0)
    }
}
