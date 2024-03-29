use super::WorldRank;
use super::WorldSize;
use crate::named::Named;
use crate::simulation::Simulation;
use crate::simulation::SubsweepPlugin;

#[derive(Clone, Named)]
pub struct BaseCommunicationPlugin {
    num_ranks: WorldSize,
    world_rank: WorldRank,
}

impl BaseCommunicationPlugin {
    pub fn new(size: usize, rank: super::Rank) -> Self {
        Self {
            num_ranks: WorldSize(size),
            world_rank: WorldRank(rank),
        }
    }
}

impl SubsweepPlugin for BaseCommunicationPlugin {
    fn build_once_everywhere(&self, sim: &mut Simulation) {
        sim.insert_resource(self.world_rank)
            .insert_resource(self.num_ranks);
    }
}
