use std::fs;
use std::time::Instant;

use bevy::prelude::NonSendMut;
use bevy::prelude::Res;
use bevy::prelude::Resource;
use serde::Serialize;
use serde_yaml::Value;

use crate::hash_map::HashMap;
use crate::io::output::parameters::OutputParameters;
use crate::units::Time;

type Category = String;

#[derive(Debug, Serialize)]
enum Result {
    RunTimes(Vec<Time>),
    Number(i32),
}

impl Result {
    fn unwrap_runtimes(&mut self) -> &mut Vec<Time> {
        if let Self::RunTimes(ref mut run_times) = self {
            run_times
        } else {
            panic!("Not a runtime statistic")
        }
    }

    fn add_timing(&mut self, elapsed_time: Time) {
        self.unwrap_runtimes().push(elapsed_time);
    }

    fn total(&self) -> Time {
        {
            if let Self::RunTimes(ref run_times) = self {
                run_times
            } else {
                panic!("Not a runtime statistic")
            }
        }
        .iter()
        .copied()
        .sum()
    }

    fn into_value(&self) -> Value {
        match self {
            Result::RunTimes(run_times) => serde_yaml::to_value(&compute_statistics(run_times)),
            Result::Number(num) => serde_yaml::to_value(&num),
        }
        .unwrap()
    }
}

fn compute_statistics(run_times: &[Time]) -> HashMap<String, Time> {
    let total = run_times.iter().copied().sum::<Time>();
    [
        ("average", total / (run_times.len() as f64)),
        ("total", total),
    ]
    .into_iter()
    .map(|(x, val)| (x.to_owned(), val))
    .collect()
}

#[derive(Debug)]
struct Timer(Instant);

impl Default for Timer {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl Timer {
    fn elapsed_time(&self) -> Time {
        Time::nanoseconds(Instant::now().duration_since(self.0).as_nanos() as f64)
    }
}

#[derive(Resource, Default, Debug, Serialize)]
pub struct Timers {
    results: HashMap<Category, Result>,
    #[serde(skip)]
    timers: HashMap<Category, Timer>,
}

impl Timers {
    pub fn start(&mut self, name: &str) {
        self.timers.insert(name.into(), Timer::default());
    }

    pub fn stop(&mut self, name: &str) {
        let name = name.into();
        let timer = self
            .timers
            .remove(&name)
            .unwrap_or_else(|| panic!("Tried to stop timer that was never started: {}", name));
        self.results
            .entry(name)
            .or_insert(Result::RunTimes(vec![]))
            .add_timing(timer.elapsed_time());
    }

    pub fn total(&self, name: &str) -> Time {
        self.results
            .get(name)
            .unwrap_or_else(|| {
                panic!(
                    "Tried to obtain performance result for non-existent category: {}",
                    name
                )
            })
            .total()
    }

    pub fn record_number(&mut self, name: &str, val: impl TryInto<i32>) {
        match val.try_into() {
            Ok(val) => self.results.insert(name.into(), Result::Number(val)),
            Err(_) => panic!(),
        };
    }

    pub(crate) fn time<'a, 'b>(&'a mut self, name: &'b str) -> TimerGuard<'a, 'b> {
        self.start(name);
        TimerGuard { data: self, name }
    }

    pub fn into_output(&self) -> HashMap<Category, Value> {
        self.results
            .iter()
            .map(|(name, result)| (name.into(), result.into_value()))
            .collect()
    }
}

#[must_use = "A timer guard needs to be used."]
pub struct TimerGuard<'a, 'b> {
    data: &'a mut Timers,
    name: &'b str,
}

impl<'a, 'b> Drop for TimerGuard<'a, 'b> {
    fn drop(&mut self) {
        self.data.stop(self.name)
    }
}

pub fn write_performance_data_system(
    timers: NonSendMut<Timers>,
    parameters: Res<OutputParameters>,
) {
    fs::write(
        parameters
            .output_dir
            .join(&parameters.performance_data_filename),
        serde_yaml::to_string(&timers.into_output()).unwrap(),
    )
    .unwrap_or_else(|e| panic!("Failed to write performance data to file. {}", e));
}
