mod camera;
mod camera_transform;
mod drawing;
pub mod parameters;
pub mod remote;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
pub use camera_transform::CameraTransform;
pub use drawing::DrawCircle;
pub use drawing::DrawRect;

use self::camera::camera_scale_system;
use self::camera::camera_translation_system;
use self::camera::setup_camera_system;
use self::drawing::draw_translation_system;
use self::drawing::DrawBundlePlugin;
use self::drawing::IntoBundle;
use self::parameters::VisualizationParameters;
use self::remote::receive_particles_on_main_thread_system;
use self::remote::send_particles_to_main_thread_system;
use self::remote::ParticleVisualizationExchangeData;
use self::remote::RemoteParticleVisualization;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Rank;
use crate::domain::determine_global_extent_system;
use crate::named::Named;
use crate::physics::gravity;
use crate::physics::LocalParticle;
use crate::physics::StopSimulationEvent;
use crate::position::Position;
use crate::quadtree::QuadTreeVisualizationPlugin;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

const COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW];

pub static CIRCLE_RADIUS: f64 = 3.0;

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

impl RaxiomPlugin for VisualizationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(
            CommunicationPlugin::<ParticleVisualizationExchangeData>::new(CommunicationType::Sync),
        );
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<VisualizationParameters>()
            .insert_resource(CameraTransform::default())
            .add_bevy_plugin(ShapePlugin)
            .add_plugin(DrawBundlePlugin::<DrawRect>::default())
            .add_plugin(DrawBundlePlugin::<DrawCircle>::default())
            .add_plugin(QuadTreeVisualizationPlugin::<
                gravity::NodeData,
                gravity::LeafData,
            >::default())
            .add_startup_system(setup_camera_system)
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                camera_scale_system.after(determine_global_extent_system),
            )
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                camera_translation_system
                    .after(determine_global_extent_system)
                    .after(camera_scale_system),
            )
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
            .add_system_to_stage(VisualizationStage::AppExit, keyboard_app_exit_system);
    }

    fn build_on_other_ranks(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(
            VisualizationStage::Synchronize,
            send_particles_to_main_thread_system,
        );
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
        commands
            .entity(entity)
            .insert(DrawCircle::from_position_and_color(
                **pos,
                colored.get_color(),
            ));
    }
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

trait GetColor {
    fn get_color(&self) -> Color;
}

impl GetColor for LocalParticle {
    fn get_color(&self) -> Color {
        get_color(0)
    }
}
