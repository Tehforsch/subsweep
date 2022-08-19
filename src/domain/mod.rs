mod extents;
pub mod quadtree;

use self::extents::Extents;
use crate::communication::DataByRank;
use crate::communication::Rank;
use crate::units::vec2;

#[derive(Clone)]
pub struct DomainDistribution {
    domains: DataByRank<Vec<Extents>>,
}

impl DomainDistribution {
    pub fn target_rank(&self, pos: &vec2::Length) -> Rank {
        *self
            .domains
            .iter()
            .find(|(_, extents)| extents.iter().any(|extent| extent.contains(pos)))
            .map(|(rank, _)| rank)
            .expect("sum of domain extents does not cover all particles")
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::extents::Extents;
    use super::DomainDistribution;
    use crate::communication::DataByRank;
    use crate::units::f32::meter;
    use crate::units::vec2;

    #[test]
    fn target_rank() {
        let total_extents = Extents::new(meter(-100.0), meter(100.0), meter(-100.0), meter(100.0));
        let quadrants = total_extents.get_quadrants();
        let mut domains = DataByRank::empty();
        domains.insert(0, vec![quadrants[0].clone(), quadrants[1].clone()]);
        domains.insert(1, vec![quadrants[2].clone(), quadrants[3].clone()]);
        let distribution = DomainDistribution { domains };
        assert_eq!(
            distribution.target_rank(&vec2::meter(Vec2::new(-70.0, -70.0))),
            0
        );
    }
}
