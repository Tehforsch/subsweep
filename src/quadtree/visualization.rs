use bevy::prelude::*;

use super::QuadTree;
use crate::domain::TopLevelIndices;
use crate::named::Named;
use crate::simulation::Simulation;
use crate::simulation::TenetPlugin;
use crate::visualization::get_color;
use crate::visualization::parameters::VisualizationParameters;
use crate::visualization::DrawRect;
use crate::visualization::VisualizationStage;

#[derive(Component)]
struct Outline;

#[derive(Named)]
pub struct QuadTreeVisualizationPlugin;

impl TenetPlugin for QuadTreeVisualizationPlugin {
    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(VisualizationStage::AddVisualization, show_quadtree_system);
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.unwrap_resource::<VisualizationParameters>()
            .show_quadtree
    }
}

fn show_quadtree_system(
    mut commands: Commands,
    quadtree: Res<QuadTree>,
    indices: Res<TopLevelIndices>,
    outlines: Query<Entity, With<Outline>>,
) {
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    for (rank, indices) in indices.iter() {
        for index in indices {
            quadtree[index].depth_first_map_leaf(&mut |extent, _| {
                commands.spawn().insert(Outline).insert(DrawRect {
                    lower_left: extent.min,
                    upper_right: extent.max,
                    color: get_color(*rank),
                });
            });
        }
    }
}
