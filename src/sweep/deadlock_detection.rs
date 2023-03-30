use bevy::prelude::warn;
use bevy::utils::StableHashSet;
use mpi::traits::Equivalence;

use super::Sweep;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::communication::MpiWorld;
use crate::grid::ParticleType;
use crate::prelude::ParticleId;

const DEADLOCK_DETECTION_TAG: i32 = 99123151;

#[derive(Clone, Equivalence, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct Dependency {
    p1: ParticleId,
    p2: ParticleId,
}

impl<'a> Sweep<'a> {
    pub fn check_deadlock(&mut self) {
        let mut dependencies: DataByRank<Vec<_>> =
            DataByRank::from_communicator(&self.communicator);

        for (id, cell) in self.cells.enumerate_active(self.current_level) {
            for (_, neighbour) in cell.neighbours.iter() {
                match neighbour {
                    ParticleType::Remote(neigh) => {
                        let rank = neigh.rank;
                        let dep = if *id < neigh.id {
                            Dependency {
                                p1: *id,
                                p2: neigh.id,
                            }
                        } else {
                            Dependency {
                                p2: *id,
                                p1: neigh.id,
                            }
                        };
                        dependencies[rank].push(dep);
                    }
                    _ => {}
                }
            }
        }
        let w = MpiWorld::new(DEADLOCK_DETECTION_TAG);
        let mut ex: ExchangeCommunicator<Dependency> = ExchangeCommunicator::from(w);
        let received = ex.exchange_all(dependencies.clone());
        warn!("Checking for deadlocks at level: {}", self.current_level.0);
        for (rank, data) in received.iter() {
            let d1: StableHashSet<_> = data.iter().cloned().collect();
            let d2: StableHashSet<_> = dependencies[*rank].iter().cloned().collect();
            if d1 != d2 {
                panic!(
                    "Found different dependencies: {}",
                    d1.symmetric_difference(&d2).count()
                );
            }
            assert_eq!(d1, d2);
        }
    }
}
