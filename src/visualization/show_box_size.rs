use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Res;

use super::DrawRect;
use super::RColor;
use super::VisualizationParameters;
use crate::named::Named;
use crate::parameters::BoxSize;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;

#[derive(Named)]
pub(super) struct ShowBoxSizePlugin;

impl RaxiomPlugin for ShowBoxSizePlugin {
    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_startup_system(show_box_size_system);
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.unwrap_resource::<VisualizationParameters>()
            .show_box_size
    }
}

#[derive(Component)]
struct BoxSizeOutline;

fn show_box_size_system(mut commands: Commands, box_size: Res<BoxSize>) {
    commands
        .spawn()
        .insert(BoxSizeOutline)
        .insert(DrawRect::from_min_max(
            box_size.min,
            box_size.max,
            RColor::BLACK,
        ));
}
