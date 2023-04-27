use bevy::ecs::schedule::StageLabelId;
use bevy::prelude::*;

use crate::domain::DomainStages;
use crate::domain::DomainStartupStages;
use crate::io::output::OutputStages;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_plugin::SimulationStages;
use crate::simulation_plugin::SimulationStartupStages;

#[derive(Named)]
pub struct SimulationStagesPlugin;

impl RaxiomPlugin for SimulationStagesPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let stages: &[StageLabelId] = &[
            CoreStage::Update.as_label(),
            DomainStages::TopLevelTreeConstruction.as_label(),
            DomainStages::Decomposition.as_label(),
            DomainStages::Exchange.as_label(),
            SimulationStages::SetTimestep.as_label(),
            SimulationStages::ForceCalculation.as_label(),
            SimulationStages::Integration.as_label(),
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
            DomainStartupStages::CheckParticleExtent.as_label(),
            DomainStartupStages::Decomposition.as_label(),
            DomainStartupStages::SetOutgoingEntities.as_label(),
            DomainStartupStages::Exchange.as_label(),
            DomainStartupStages::ParticleIds.as_label(),
            DomainStartupStages::TreeConstruction.as_label(),
            SimulationStartupStages::InsertGrid.as_label(),
            SimulationStartupStages::InsertComponentsAfterGrid.as_label(),
            SimulationStartupStages::Sweep.as_label(),
            SimulationStartupStages::Final.as_label(),
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
