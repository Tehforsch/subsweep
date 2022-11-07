mod camera;
mod camera_transform;
pub(super) mod color;
mod draw_item;
pub mod parameters;
pub mod remote;
mod show_halo_particles;
mod show_particles;

use bevy::prelude::*;
pub use camera_transform::CameraTransform;
pub use draw_item::circle::DrawCircle;
pub use draw_item::rect::DrawRect;

use self::camera::camera_scale_system;
use self::camera::camera_translation_system;
use self::camera::setup_camera_system;
pub use self::color::RColor;
pub use self::draw_item::DrawItem;
use self::draw_item::DrawItemPlugin;
pub use self::draw_item::Pixels;
pub use self::parameters::VisualizationParameters;
use self::show_halo_particles::ShowHaloParticlesPlugin;
pub use self::show_particles::ShowParticlesPlugin;
use crate::domain::determine_global_extent_system;
use crate::gravity;
use crate::named::Named;
use crate::quadtree::QuadTreeVisualizationPlugin;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_plugin::StopSimulationEvent;

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
    fn build_always_once(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<VisualizationParameters>();
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(DrawItemPlugin::<DrawRect>::default())
            .add_plugin(DrawItemPlugin::<DrawCircle>::default())
            .add_plugin(ShowParticlesPlugin)
            .add_plugin(ShowHaloParticlesPlugin);
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.insert_resource(CameraTransform::default())
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
            .add_system_to_stage(VisualizationStage::AppExit, keyboard_app_exit_system);
        sim.add_plugin(QuadTreeVisualizationPlugin::<
            gravity::NodeData,
            gravity::LeafData,
        >::default());
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
