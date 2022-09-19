mod drawing;
pub mod parameters;
pub mod remote;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
pub use drawing::DrawCircle;
pub use drawing::DrawRect;
use mpi::traits::Equivalence;

use self::drawing::draw_translation_system;
use self::drawing::DrawBundlePlugin;
use self::drawing::IntoBundle;
use self::parameters::Parameters;
use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use self::remote::ParticleVisualizationExchangeData;
use self::remote::RemoteParticleVisualization;
use crate::communication::AllGatherCommunicator;
use crate::communication::CollectiveCommunicator;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Rank;
use crate::communication::WorldRank;
use crate::domain::GlobalExtent;
use crate::parameters::ParameterPlugin;
use crate::physics::LocalParticle;
use crate::physics::StopSimulationEvent;
use crate::position::Position;
use crate::quadtree::QuadTreeVisualizationPlugin;

const COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW];

pub static CIRCLE_RADIUS: f64 = 3.0;

#[derive(Equivalence, Clone)]
struct ShouldExit(bool);

#[derive(StageLabel)]
pub enum VisualizationStage {
    Synchronize,
    AddVisualization,
    AddDrawComponents,
    Draw,
    AppExit,
}

pub struct VisualizationPlugin;

#[derive(Component)]
struct WorldCamera;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        let rank = *app.world.get_resource::<WorldRank>().unwrap();
        app.add_plugin(
            CommunicationPlugin::<ParticleVisualizationExchangeData>::new(CommunicationType::Sync),
        )
        .add_plugin(CommunicationPlugin::<ShouldExit>::new(
            CommunicationType::AllGather,
        ));

        if rank.is_main() {
            app.add_plugin(ParameterPlugin::<Parameters>::new("visualization"))
                .add_plugin(ShapePlugin)
                .add_plugin(DrawBundlePlugin::<DrawRect>::default())
                .add_plugin(DrawBundlePlugin::<DrawCircle>::default())
                .add_plugin(QuadTreeVisualizationPlugin)
                .add_startup_system(setup_camera_system)
                .add_startup_system_to_stage(StartupStage::PostStartup, camera_translation_system)
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
                )
                .add_system_to_stage(VisualizationStage::AppExit, keyboard_app_exit_system)
                .add_system_to_stage(
                    VisualizationStage::AppExit,
                    handle_app_exit_system.after(keyboard_app_exit_system),
                );
        } else {
            app.add_system_to_stage(
                VisualizationStage::Synchronize,
                send_particles_to_main_thread_system,
            )
            .add_system_to_stage(VisualizationStage::AppExit, handle_app_exit_system);
        }
    }
}

fn camera_translation_system(
    mut camera: Query<&mut Transform, With<WorldCamera>>,
    extent: Res<GlobalExtent>,
    parameters: Res<Parameters>,
) {
    let mut camera_transform = camera.single_mut();
    let pos = extent.center.in_units(parameters.camera_zoom);
    camera_transform.translation.x = pos.x as f32;
    camera_transform.translation.y = pos.y as f32;
}

pub fn get_color(rank: Rank) -> Color {
    COLORS[(rank as usize).rem_euclid(COLORS.len())]
}

fn spawn_sprites_system<T: Component + GetColor>(
    mut commands: Commands,
    particles: Query<(Entity, &Position, &T), (With<T>, Without<DrawCircle>)>,
    parameters: Res<Parameters>,
) {
    for (entity, pos, colored) in particles.iter() {
        commands.entity(entity).insert(DrawCircle {
            position: **pos,
            radius: CIRCLE_RADIUS * parameters.camera_zoom,
            color: colored.get_color(),
        });
    }
}

pub fn setup_camera_system(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(WorldCamera);
}

fn position_to_translation_system<T: Component + IntoBundle>(
    mut query: Query<(&mut T, &Position)>,
) {
    for (mut item, position) in query.iter_mut() {
        item.set_translation(position);
    }
}

fn keyboard_app_exit_system(
    input: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<StopSimulationEvent>,
) {
    if input.just_pressed(KeyCode::Escape) {
        event_writer.send(StopSimulationEvent);
    }
}

fn handle_app_exit_system(
    mut event_reader: EventReader<StopSimulationEvent>,
    mut event_writer: EventWriter<AppExit>,
    mut comm: NonSendMut<AllGatherCommunicator<ShouldExit>>,
) {
    let result = if event_reader.iter().count() > 0 {
        comm.all_gather(&ShouldExit(true))
    } else {
        comm.all_gather(&ShouldExit(false))
    };
    let should_exit = result.into_iter().any(|x| x.0);
    if should_exit {
        event_writer.send(AppExit);
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
