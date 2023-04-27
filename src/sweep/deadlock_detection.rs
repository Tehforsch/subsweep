use bevy::prelude::warn;
use mpi::traits::Equivalence;

use super::chemistry::Chemistry;
use super::timestep_level::TimestepLevel;
use super::Sweep;
use crate::communication::communicator::Communicator;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::communication::MpiWorld;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::grid::ParticleType;
use crate::hash_map::HashSet;
use crate::prelude::ParticleId;

const DEADLOCK_DETECTION_TAG: i32 = 99123151;

#[derive(Clone, Equivalence, PartialOrd, Ord, Debug, PartialEq, Eq, Hash)]
struct Dependency {
    p1: ParticleInfo,
    p2: ParticleInfo,
}

#[derive(Clone, Equivalence, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct ParticleInfo {
    rank: Rank,
    id: ParticleId,
    level: TimestepLevel,
}

impl std::fmt::Display for ParticleInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(rank={:>3} id={:>6} level={:>2})",
            self.rank, self.id.index, self.level.0
        )
    }
}

impl<C: Chemistry> Sweep<C> {
    fn get_dependency(
        &self,
        p1: ParticleId,
        rank1: Rank,
        p2: ParticleId,
        rank2: Rank,
    ) -> Dependency {
        Dependency {
            p1: ParticleInfo {
                id: p1,
                level: self.levels[&p1],
                rank: rank1,
            },
            p2: ParticleInfo {
                id: p2,
                level: self.levels[&p2],
                rank: rank2,
            },
        }
    }

    fn get_dependencies(&self) -> DataByRank<Vec<Dependency>> {
        let mut dependencies: DataByRank<Vec<_>> =
            DataByRank::from_communicator(&self.communicator);

        for (id, cell) in self.cells.enumerate_active(self.current_level) {
            for (_, neighbour) in cell.neighbours.iter() {
                if let ParticleType::Remote(neigh) = neighbour {
                    assert!(self.is_active(*id));
                    if self.is_active(neigh.id) {
                        let dep = if neigh.rank > self.communicator.rank() {
                            self.get_dependency(*id, self.communicator.rank(), neigh.id, neigh.rank)
                        } else {
                            self.get_dependency(neigh.id, neigh.rank, *id, self.communicator.rank())
                        };
                        dependencies[neigh.rank].push(dep);
                    }
                }
            }
        }
        dependencies
    }

    fn check_some_initial_task_exists(&self) {
        let num_to_solve = self.cells.enumerate_active(self.current_level).count();
        if num_to_solve == 0 {
            return;
        }
        let num_initial_tasks = self.to_solve.len();
        let w = MpiWorld::new_custom_tag(DEADLOCK_DETECTION_TAG);
        let mut ex: Communicator<usize> = Communicator::from(w);
        let total: usize = ex.all_gather_sum(&num_initial_tasks);
        assert!(
            total > 0,
            "No rank has an initial task. Wrong boundary setup?"
        )
    }

    pub fn check_deadlock(&mut self) {
        self.check_some_initial_task_exists();
        let dependencies = self.get_dependencies();
        let w = MpiWorld::new_custom_tag(DEADLOCK_DETECTION_TAG);
        let mut ex: ExchangeCommunicator<Dependency> = ExchangeCommunicator::from(w);
        let received = ex.exchange_all(dependencies.clone());
        warn!("Checking for deadlocks at level: {}", self.current_level.0);
        for (rank, data) in received.iter() {
            let d1: HashSet<_> = data.iter().cloned().collect();
            let d2: HashSet<_> = dependencies[*rank].iter().cloned().collect();
            if d1 != d2 {
                if self.communicator.rank() < *rank {
                    println!("On rank {}:", self.communicator.rank());
                    print_diff(&d1, &d2);
                    println!("On rank {}:", rank);
                    print_diff(&d2, &d1);
                }
                panic!(
                    "Found {} different dependencies",
                    d1.symmetric_difference(&d2).count()
                );
            }
        }
    }
}

fn print_diff(set1: &HashSet<Dependency>, set2: &HashSet<Dependency>) {
    let mut diff: Vec<_> = set1.difference(set2).cloned().collect();
    diff.sort();
    for dep in diff.into_iter() {
        println!("{:<6} <-> {:<6}", dep.p1, dep.p2);
    }
}
