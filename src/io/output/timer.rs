use bevy_ecs::prelude::Commands;
use bevy_ecs::prelude::EventReader;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::ResMut;
use bevy_ecs::prelude::Resource;
use bevy_ecs::schedule::ShouldRun;

use super::parameters::OutputParameters;
use crate::simulation_plugin::SimulationTime;
use crate::simulation_plugin::StopSimulationEvent;
use crate::units;

#[derive(Resource)]
pub struct Timer {
    next_output_time: units::Time,
    snapshot_num: usize,
}

impl Timer {
    pub fn initialize_system(mut commands: Commands, parameters: Res<OutputParameters>) {
        commands.insert_resource(Timer {
            next_output_time: parameters
                .time_first_snapshot
                .unwrap_or_else(units::Time::zero),
            snapshot_num: 0,
        });
    }

    pub fn run_criterion(
        time: Res<SimulationTime>,
        timer: Res<Self>,
        events: EventReader<StopSimulationEvent>,
    ) -> ShouldRun {
        let simulation_finished = !events.is_empty();
        if simulation_finished || time.0 >= timer.next_output_time {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }

    pub fn update_system(mut output_timer: ResMut<Self>, parameters: Res<OutputParameters>) {
        output_timer.snapshot_num += 1;
        output_timer.next_output_time += parameters.time_between_snapshots;
    }

    pub fn snapshot_num(&self) -> usize {
        self.snapshot_num
    }

    pub fn is_first_snapshot(&self) -> bool {
        self.snapshot_num == 0
    }
}
