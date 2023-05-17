use bevy::ecs::schedule::StageLabelId;
use bevy::prelude::*;

use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_plugin::Stages;
use crate::simulation_plugin::StartupStages;

#[derive(Named)]
pub struct SimulationStagesPlugin;

impl RaxiomPlugin for SimulationStagesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let stages: &[StageLabelId] = &[
            CoreStage::Update.as_label(),
            Stages::Initial.as_label(),
            Stages::ForceCalculation.as_label(),
            Stages::Integration.as_label(),
            Stages::Output.as_label(),
            Stages::Final.as_label(),
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
            StartupStages::InsertComponents.as_label(),
            StartupStages::InsertDerivedComponents.as_label(),
            StartupStages::CheckParticleExtent.as_label(),
            StartupStages::Decomposition.as_label(),
            StartupStages::SetOutgoingEntities.as_label(),
            StartupStages::Exchange.as_label(),
            StartupStages::ParticleIds.as_label(),
            StartupStages::TreeConstruction.as_label(),
            StartupStages::InsertGrid.as_label(),
            StartupStages::InsertComponentsAfterGrid.as_label(),
            StartupStages::Sweep.as_label(),
            StartupStages::Final.as_label(),
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
