use std::iter;

use bevy_app::CoreStage;
use bevy_app::StartupSchedule;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ShouldRun;
use bevy_ecs::schedule::StageLabelId;

use crate::performance::Performance;
use crate::simulation_plugin::Stages;
use crate::simulation_plugin::StartupStages;

#[derive(StageLabel, Clone, Copy)]
enum TimerStages {
    Timer1,
    Timer2,
    Timer3,
    Timer4,
    Timer5,
    Timer6,
    Timer7,
    Timer8,
    Timer9,
    Timer10,
    Timer11,
    Timer12,
    Timer13,
    Timer14,
}
impl TimerStages {
    fn all() -> impl Iterator<Item = Self> {
        [
            Self::Timer1,
            Self::Timer2,
            Self::Timer3,
            Self::Timer4,
            Self::Timer5,
            Self::Timer6,
            Self::Timer7,
            Self::Timer8,
            Self::Timer9,
            Self::Timer10,
            Self::Timer11,
            Self::Timer12,
            Self::Timer13,
            Self::Timer14,
        ]
        .into_iter()
    }
}

fn get_stages() -> [StageLabelId; 7] {
    [
        CoreStage::First.as_label(),
        Stages::Initial.as_label(),
        Stages::Sweep.as_label(),
        Stages::AfterSweep.as_label(),
        Stages::Output.as_label(),
        Stages::Final.as_label(),
        CoreStage::Last.as_label(),
    ]
}

fn get_startup_stages() -> [StageLabelId; 13] {
    [
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
    ]
}

fn add_timers(stages: &[StageLabelId]) -> Vec<StageLabelId> {
    let timers = TimerStages::all().collect::<Vec<_>>();
    let timers = timers.iter();
    assert!(
        timers.len() > stages.len(),
        "Too many stages for the amount of timer stages, add more entries in the enum"
    );
    let mut stages: Vec<_> = timers
        .zip(stages)
        .flat_map(|(timer, stage)| iter::once(timer.as_label()).chain(iter::once(stage.as_label())))
        .collect();
    stages.push(TimerStages::all().last().unwrap().as_label());
    stages
}

pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    let mut startup_schedule = Schedule::default().with_run_criteria(ShouldRun::once);
    make_schedule_from_stage_labels(&mut startup_schedule, &add_timers(&get_startup_stages()));
    schedule.add_stage(StartupSchedule, startup_schedule);
    make_schedule_from_stage_labels(&mut schedule, &add_timers(&get_stages()));
    schedule
}

fn make_schedule_from_stage_labels(schedule: &mut Schedule, labels: &[StageLabelId]) {
    for stage in labels {
        schedule.add_stage(stage.as_label(), SystemStage::single_threaded());
    }
    add_stage_timers_for_stages(schedule, labels);
}

fn add_stage_timers_for_stages(schedule: &mut Schedule, stages: &[StageLabelId]) {
    let is_timer_stage =
        |stage: StageLabelId| TimerStages::all().any(|timer_stage| stage == timer_stage.as_label());
    for window in stages.windows(3) {
        let (s1, s2, s3) = (window[0], window[1], window[2]);
        if !is_timer_stage(s2) {
            schedule.add_system_to_stage(s1, move |mut timers: ResMut<Performance>| {
                timers.start(s2.as_str());
            });
            schedule.add_system_to_stage(s3, move |mut timers: ResMut<Performance>| {
                timers.stop(s2.as_str());
            });
        }
    }
}
