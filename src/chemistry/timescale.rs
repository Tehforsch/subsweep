use bevy::prelude::debug;

use crate::hash_map::HashMap;
use crate::sweep::timestep_level::TimestepLevel;
use crate::units::Time;

#[derive(Clone, Copy)]
pub struct Timescale {
    pub time: Time,
    pub process: Process,
}

impl Timescale {
    pub fn ionization_fraction(time: Time) -> Self {
        Self {
            time,
            process: Process::IonizationFraction,
        }
    }
    pub fn temperature(time: Time) -> Self {
        Self {
            time,
            process: Process::Temperature,
        }
    }
    pub fn photon_rate(time: Time) -> Self {
        Self {
            time,
            process: Process::PhotonRate,
        }
    }

    pub fn min(&self, other: Self) -> Self {
        if self.time < other.time {
            *self
        } else {
            other
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Process {
    Temperature,
    IonizationFraction,
    PhotonRate,
}

impl Process {
    pub(crate) fn iter_all() -> impl Iterator<Item = Self> {
        [
            Self::Temperature,
            Self::IonizationFraction,
            Self::PhotonRate,
        ]
        .into_iter()
    }
}

impl std::fmt::Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Process::Temperature => "temperature",
            Process::IonizationFraction => "ionization fraction",
            Process::PhotonRate => "photon rate",
        };
        write!(f, "{}", s)
    }
}

pub struct TimescaleCounter {
    limiting_processes: HashMap<Process, usize>,
    max_timestep: Time,
}

impl TimescaleCounter {
    pub fn new(max_timestep: Time) -> Self {
        Self {
            limiting_processes: Process::iter_all().map(|process| (process, 0)).collect(),
            max_timestep,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.max_timestep);
    }

    pub fn show_timestep_limiting_processes(&mut self, current_level: TimestepLevel) {
        if current_level.is_highest_timestep() {
            self.show_statistics();
        }
        self.reset();
    }

    pub fn count(&mut self, change_timescale: Timescale) {
        if change_timescale.time < self.max_timestep {
            *self
                .limiting_processes
                .get_mut(&change_timescale.process)
                .unwrap() += 1;
        }
    }

    fn show_statistics(&self) {
        let total: usize = self.limiting_processes.values().sum();
        if total == 0 {
            return;
        }
        let mut processes: Vec<_> = Process::iter_all().collect();
        processes.sort_by_key(|process| self.limiting_processes[process]);
        for process in processes {
            let percentage = 100.0 * self.limiting_processes[&process] as f64 / total as f64;
            debug!(
                "Timestep of {:>5.1}% of particles limited by {}",
                percentage, process
            )
        }
    }
}
