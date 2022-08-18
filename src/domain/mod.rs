use std::collections::HashMap;

use glam::Vec2;

use crate::communication::Rank;
use crate::position::Position;
use crate::units::vec2::meter;
use crate::units::vec2::Length;

#[derive(Clone)]
pub struct Domain {
    pub upper_left: Length,
    pub lower_right: Length,
}

impl Domain {
    fn contains(&self, pos: &Position) -> bool {
        let ul = self.upper_left.unwrap_value();
        let lr = self.lower_right.unwrap_value();
        let pos = pos.0.unwrap_value();
        ul.x <= pos.x && pos.x < lr.x && ul.y <= pos.y && pos.y < lr.y
    }
}

#[derive(Clone)]
pub struct DomainDistribution {
    pub domains: HashMap<Rank, Domain>,
}

impl DomainDistribution {
    pub fn target_rank(&self, pos: &Position) -> Rank {
        *self
            .domains
            .iter()
            .find(|(_, domain)| domain.contains(pos))
            .map(|(rank, _)| rank)
            .unwrap_or(&0)
    }
}

pub fn get_domain_distribution() -> DomainDistribution {
    DomainDistribution {
        domains: [
            (
                0,
                Domain {
                    upper_left: meter(Vec2::new(-100.0, -100.0)),
                    lower_right: meter(Vec2::new(0.0, 0.0)),
                },
            ),
            (
                1,
                Domain {
                    upper_left: meter(Vec2::new(0.0, -100.0)),
                    lower_right: meter(Vec2::new(100.0, 0.0)),
                },
            ),
            (
                2,
                Domain {
                    upper_left: meter(Vec2::new(0.0, 0.0)),
                    lower_right: meter(Vec2::new(100.0, 100.0)),
                },
            ),
            (
                3,
                Domain {
                    upper_left: meter(Vec2::new(-100.0, 0.0)),
                    lower_right: meter(Vec2::new(0.0, 100.0)),
                },
            ),
        ]
        .into_iter()
        .collect(),
    }
}
