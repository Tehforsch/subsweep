mod extents;
pub mod quadtree;

use std::collections::HashMap;

use self::extents::Extents;
use crate::communication::Rank;
use crate::position::Position;
use crate::units::f32::meter;

#[derive(Clone)]
pub struct DomainDistribution {
    pub domains: HashMap<Rank, Extents>,
}

impl DomainDistribution {
    pub fn target_rank(&self, pos: &Position) -> Rank {
        *self
            .domains
            .iter()
            .find(|(_, domain)| domain.contains(&pos.0))
            .map(|(rank, _)| rank)
            .unwrap_or(&0)
    }
}

pub fn get_domain_distribution() -> DomainDistribution {
    let total_extents = Extents::new(meter(-100.0), meter(100.0), meter(-100.0), meter(100.0));
    return DomainDistribution {
        domains: [(0, total_extents)].into_iter().collect(),
    };
    // let quadrants = total_extents.get_quadrants();
    // DomainDistribution {
    //     domains: quadrants
    //         .into_iter()
    //         .enumerate()
    //         .map(|(i, quadrant)| (i as Rank, quadrant))
    //         .collect(),
    // }
}
