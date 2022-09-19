use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::Commands;
use bevy::prelude::EventReader;
use bevy::prelude::Res;
use bevy::prelude::ResMut;

use super::parameters::Parameters;
use crate::physics::StopSimulationEvent;
use crate::physics::Time;
use crate::units;

pub(super) struct Timer {
    next_output_time: units::Time,
    snapshot_num: usize,
}

impl Timer {
    pub fn initialize_system(mut commands: Commands, parameters: Res<Parameters>) {
        commands.insert_resource(Timer {
            next_output_time: parameters
                .time_first_snapshot
                .unwrap_or_else(units::Time::zero),
            snapshot_num: 0,
        });
    }

    pub fn run_criterion(
        time: Res<Time>,
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

    pub fn update_system(mut output_timer: ResMut<Self>, parameters: Res<Parameters>) {
        output_timer.snapshot_num += 1;
        output_timer.next_output_time += parameters.time_between_snapshots;
    }

    pub fn snapshot_num(&self) -> usize {
        self.snapshot_num
    }
}
