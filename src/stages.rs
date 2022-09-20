use bevy::ecs::schedule::StageLabelId;
use bevy::prelude::*;

use crate::domain::DomainDecompositionStages;
use crate::io::output::OutputStages;
use crate::named::Named;
use crate::physics::hydrodynamics::HydrodynamicsStages;
use crate::physics::PhysicsStages;
use crate::plugin_utils::Simulation;
use crate::plugin_utils::TenetPlugin;
use crate::visualization::VisualizationStage;

pub struct SimulationStagesPlugin;

impl Named for SimulationStagesPlugin {
    fn name() -> &'static str {
        "simulation_stages"
    }
}

impl TenetPlugin for SimulationStagesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let stages: &[StageLabelId] = &[
            CoreStage::Update.as_label(),
            DomainDecompositionStages::TopLevelTreeConstruction.as_label(),
            DomainDecompositionStages::Decomposition.as_label(),
            DomainDecompositionStages::Exchange.as_label(),
            HydrodynamicsStages::Hydrodynamics.as_label(),
            PhysicsStages::Physics.as_label(),
            VisualizationStage::Synchronize.as_label(),
            VisualizationStage::AddVisualization.as_label(),
            VisualizationStage::AddDrawComponents.as_label(),
            VisualizationStage::Draw.as_label(),
            VisualizationStage::AppExit.as_label(),
            OutputStages::Output.as_label(),
        ];
        for window in stages.windows(2) {
            sim.add_stage_after(
                window[0].as_label(),
                window[1].as_label(),
                SystemStage::parallel(),
            );
        }
    }
}
