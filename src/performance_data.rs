use std::fs;
use std::time::Instant;

use bevy_ecs::prelude::NonSendMut;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::Resource;
use linked_hash_map::LinkedHashMap;
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

    fn as_value(&self) -> Value {
        match self {
            Result::RunTimes(run_times) => serde_yaml::to_value(Statistics::new(run_times)),
            Result::Number(num) => serde_yaml::to_value(num),
        }
        .unwrap()
    }
}

#[derive(Serialize)]
struct Statistics {
    average: Time,
    total: Time,
    num_calls: usize,
}

impl Statistics {
    fn new(run_times: &[Time]) -> Self {
        let total = run_times.iter().copied().sum::<Time>();
        let num_calls = run_times.len();
        Self {
            total,
            average: total / num_calls as f64,
            num_calls,
        }
    }
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
    pub fn start<N: Into<String>>(&mut self, name: N) {
        self.timers.insert(name.into(), Timer::default());
    }

    pub fn stop<N: Into<String>>(&mut self, name: N) {
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

    pub fn total<N: Clone + Into<String>>(&self, name: N) -> Time {
        self.results
            .get(&name.clone().into())
            .unwrap_or_else(|| {
                panic!(
                    "Tried to obtain performance result for non-existent category: {}",
                    &name.into()
                )
            })
            .total()
    }

    pub fn record_number<N: Into<String>>(&mut self, name: N, val: impl TryInto<i32>) {
        match val.try_into() {
            Ok(val) => self.results.insert(name.into(), Result::Number(val)),
            Err(_) => panic!(),
        };
    }

    pub(crate) fn time<N: Into<String> + Clone>(&mut self, name: N) -> TimerGuard<'_, N> {
        self.start(name.clone());
        TimerGuard { data: self, name }
    }

    pub fn as_output(&self) -> LinkedHashMap<Category, Value> {
        let mut names: Vec<_> = self.results.iter().map(|(name, _)| name.clone()).collect();
        names.sort();
        names
            .into_iter()
            .map(move |name| {
                let result = self.results[&name].as_value();
                (name, result)
            })
            .collect()
    }
}

#[must_use = "A timer guard needs to be used."]
pub struct TimerGuard<'a, N: Into<String> + Clone> {
    data: &'a mut Timers,
    name: N,
}

impl<'a, N: Into<String> + Clone> Drop for TimerGuard<'a, N> {
    fn drop(&mut self) {
        self.data.stop(self.name.clone())
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
        serde_yaml::to_string(&timers.as_output()).unwrap(),
    )
    .unwrap_or_else(|e| panic!("Failed to write performance data to file. {}", e));
}
