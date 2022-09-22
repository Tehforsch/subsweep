mod camera_transform;
mod drawing;
pub mod parameters;
pub mod remote;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
pub use camera_transform::CameraTransform;
pub use drawing::DrawCircle;
pub use drawing::DrawRect;
use mpi::traits::Equivalence;

use self::drawing::draw_translation_system;
use self::drawing::DrawBundlePlugin;
use self::drawing::IntoBundle;
use self::parameters::VisualizationParameters;
use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use self::remote::ParticleVisualizationExchangeData;
use self::remote::RemoteParticleVisualization;
use crate::communication::AllGatherCommunicator;
use crate::communication::CollectiveCommunicator;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Rank;
use crate::domain::determine_global_extent_system;
use crate::domain::GlobalExtent;
use crate::named::Named;
use crate::physics::LocalParticle;
use crate::physics::StopSimulationEvent;
use crate::position::Position;
use crate::quadtree::QuadTreeVisualizationPlugin;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

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

#[derive(Named)]
pub struct VisualizationPlugin;

#[derive(Component)]
struct WorldCamera;

impl RaxiomPlugin for VisualizationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(
            CommunicationPlugin::<ParticleVisualizationExchangeData>::new(CommunicationType::Sync),
        )
        .add_plugin(CommunicationPlugin::<ShouldExit>::new(
            CommunicationType::AllGather,
        ));
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<VisualizationParameters>()
            .insert_resource(CameraTransform::default())
            .add_bevy_plugin(ShapePlugin)
            .add_plugin(DrawBundlePlugin::<DrawRect>::default())
            .add_plugin(DrawBundlePlugin::<DrawCircle>::default())
            .add_plugin(QuadTreeVisualizationPlugin)
            .add_startup_system(setup_camera_system)
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                camera_scale_system.after(determine_global_extent_system),
            )
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
    }

    fn build_on_other_ranks(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(
            VisualizationStage::Synchronize,
            send_particles_to_main_thread_system,
        )
        .add_system_to_stage(VisualizationStage::AppExit, handle_app_exit_system);
    }
}

fn camera_translation_system(
    mut camera: Query<&mut Transform, With<WorldCamera>>,
    extent: Res<GlobalExtent>,
    camera_transform: Res<CameraTransform>,
) {
    let mut camera = camera.single_mut();
    let pos = camera_transform.position_to_pixels(extent.center);
    camera.translation.x = pos.x;
    camera.translation.y = pos.y;
}

fn camera_scale_system(
    extent: Res<GlobalExtent>,
    mut camera_transform: ResMut<CameraTransform>,
    windows: Res<Windows>,
) {
    let length = extent.max_side_length();
    let window = windows.primary();
    let max_side = window.width().max(window.height()).min(1000.0);
    *camera_transform = CameraTransform::from_scale(length / (max_side as f64));
}

pub fn get_color(rank: Rank) -> Color {
    COLORS[(rank as usize).rem_euclid(COLORS.len())]
}

fn spawn_sprites_system<T: Component + GetColor>(
    mut commands: Commands,
    particles: Query<(Entity, &Position, &T), (With<T>, Without<DrawCircle>)>,
) {
    for (entity, pos, colored) in particles.iter() {
        commands
            .entity(entity)
            .insert(DrawCircle::from_position_and_color(
                **pos,
                colored.get_color(),
            ));
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
    if input.just_pressed(KeyCode::Escape) && input.get_pressed().len() == 1 {
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
