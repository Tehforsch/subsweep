use bevy::prelude::*;

use super::QuadTree;
use crate::domain::TopLevelIndices;
use crate::visualization::get_color;
use crate::visualization::parameters::Parameters;
use crate::visualization::DrawRect;
use crate::visualization::VisualizationStage;

#[derive(Component)]
struct Outline;

pub struct QuadTreeVisualizationPlugin;

impl Plugin for QuadTreeVisualizationPlugin {
    fn build(&self, app: &mut App) {
        if app
            .world
            .get_resource::<Parameters>()
            .unwrap()
            .show_quadtree
        {
            app.add_system_to_stage(VisualizationStage::AddVisualization, show_quadtree_system);
        }
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
