use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::schedule::StageLabelId;
use bevy::prelude::*;

use crate::simulation_plugin::Stages;
use crate::simulation_plugin::StartupStages;

pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    let stages: &[StageLabelId] = &[
        CoreStage::First.as_label(),
        Stages::Initial.as_label(),
        Stages::Sweep.as_label(),
        Stages::AfterSweep.as_label(),
        Stages::Output.as_label(),
        Stages::Final.as_label(),
        CoreStage::Last.as_label(),
    ];
    let startup_stages: &[StageLabelId] = &[
        StartupStages::Initial.as_label(),
        StartupStages::ReadInput.as_label(),
        StartupStages::InsertDerivedComponents.as_label(),
        StartupStages::Decomposition.as_label(),
        StartupStages::SetOutgoingEntities.as_label(),
        StartupStages::Exchange.as_label(),
        StartupStages::AssignParticleIds.as_label(),
        StartupStages::TreeConstruction.as_label(),
        StartupStages::Remap.as_label(),
        StartupStages::InsertGrid.as_label(),
        StartupStages::InsertComponentsAfterGrid.as_label(),
        StartupStages::InitSweep.as_label(),
        StartupStages::Final.as_label(),
    ];
    let mut startup_schedule = Schedule::default().with_run_criteria(ShouldRun::once);
    make_schedule_from_stage_labels(&mut startup_schedule, &startup_stages);
    schedule.add_stage(StartupSchedule, startup_schedule);
    make_schedule_from_stage_labels(&mut schedule, &stages);
    schedule
}

fn make_schedule_from_stage_labels(schedule: &mut Schedule, labels: &[StageLabelId]) {
    for stage in labels {
        schedule.add_stage(stage.as_label(), SystemStage::single_threaded());
    }
}
