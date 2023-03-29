use bevy::ecs::schedule::StageLabelId;
use bevy::prelude::*;

use crate::domain::DomainDecompositionStages;
use crate::domain::DomainDecompositionStartupStages;
use crate::io::output::OutputStages;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_plugin::SimulationStages;
use crate::simulation_plugin::SimulationStartupStages;
use crate::visualization::VisualizationStage;

#[derive(Named)]
pub struct SimulationStagesPlugin;

impl RaxiomPlugin for SimulationStagesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let stages: &[StageLabelId] = &[
            CoreStage::Update.as_label(),
            DomainDecompositionStages::TopLevelTreeConstruction.as_label(),
            DomainDecompositionStages::Decomposition.as_label(),
            DomainDecompositionStages::Exchange.as_label(),
            SimulationStages::SetTimestep.as_label(),
            SimulationStages::ForceCalculation.as_label(),
            SimulationStages::Integration.as_label(),
            VisualizationStage::AddVisualization.as_label(),
            VisualizationStage::ModifyVisualization.as_label(),
            VisualizationStage::Synchronize.as_label(),
            VisualizationStage::AddDrawComponentsOnMainRank.as_label(),
            VisualizationStage::DrawOnMainRank.as_label(),
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
        let startup_stages = &[
            StartupStage::PostStartup.as_label(),
            SimulationStartupStages::InsertComponents.as_label(),
            SimulationStartupStages::InsertDerivedComponents.as_label(),
            DomainDecompositionStartupStages::DetermineGlobalExtents.as_label(),
            DomainDecompositionStartupStages::TopLevelTreeConstruction.as_label(),
            DomainDecompositionStartupStages::Decomposition.as_label(),
            DomainDecompositionStartupStages::Exchange.as_label(),
            SimulationStartupStages::InsertGrid.as_label(),
        ];
        for window in startup_stages.windows(2) {
            sim.add_startup_stage_after(
                window[0].as_label(),
                window[1].as_label(),
                SystemStage::parallel(),
            );
        }
    }
}
