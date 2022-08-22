use bevy::prelude::debug;
use mpi::traits::Equivalence;

use super::peano_hilbert::PeanoHilbertKey;
use super::ParticleData;
use crate::communication::DataByRank;

/// A segment of peano hilbert keys corresponding to
/// the interval including `start` but excluding `end`
#[derive(Debug, Equivalence, Clone)]
pub struct Segment {
    start: PeanoHilbertKey,
    end: PeanoHilbertKey,
    pub num_particles: usize,
}

fn get_position(particles: &[ParticleData], key: &PeanoHilbertKey) -> usize {
    particles
        .binary_search_by_key(key, |p: &ParticleData| p.key)
        .unwrap_or_else(|insertion_index| insertion_index)
}

fn num_contained_particles(
    particles: &[ParticleData],
    start: PeanoHilbertKey,
    end: PeanoHilbertKey,
) -> usize {
    get_position(particles, &end) - get_position(particles, &start)
}

impl Segment {
    fn new(particles: &[ParticleData], start: PeanoHilbertKey, end: PeanoHilbertKey) -> Self {
        debug_assert!(start <= end);
        Self {
            start,
            end,
            num_particles: num_contained_particles(particles, start, end),
        }
    }

    pub(super) fn iter_contained_particles<'a>(
        &self,
        particles: &'a [ParticleData],
    ) -> &'a [ParticleData] {
        &particles[get_position(&particles, &self.start)..get_position(&particles, &self.end)]
    }

    fn split_into(
        self,
        segments: &mut Vec<Segment>,
        particles: &[ParticleData],
        desired_segment_size: usize,
    ) {
        if self.num_particles == 0 {
            return;
        } else if self.num_particles > desired_segment_size {
            let half = PeanoHilbertKey((self.end.0 + self.start.0) / 2);
            let left = Segment::new(particles, self.start, half);
            let right = Segment::new(particles, half, self.end);
            left.split_into(segments, particles, desired_segment_size);
            right.split_into(segments, particles, desired_segment_size);
        } else {
            segments.push(self);
        }
    }
}

pub(super) fn get_segments(
    particles: &[ParticleData],
    desired_segment_size: usize,
) -> Vec<Segment> {
    if particles.len() == 0 {
        return vec![];
    }
    if particles.len() == 1 {
        return vec![Segment {
            start: particles[0].key,
            end: particles[0].key,
            num_particles: 1,
        }];
    }
    let segment = Segment {
        start: particles[0].key,
        end: particles.last().unwrap().key.next(),
        num_particles: particles.len(),
    };
    let mut segments = vec![];
    segment.split_into(&mut segments, &particles, desired_segment_size);
    segments
}

pub(super) fn sort_and_merge_segments(mut segments: DataByRank<Vec<Segment>>) -> Vec<Segment> {
    let mut result = vec![];
    for (_, segments) in segments.drain_all() {
        for segment in segments {
            let start = result
                .binary_search_by_key(&segment.start, |s: &Segment| s.end)
                .unwrap_or_else(|i| i);
            let end = result
                .binary_search_by_key(&segment.end, |s| s.start)
                .unwrap_or_else(|i| i);
            result.push(segment);
            debug!("for tomorrow");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use crate::domain::peano_hilbert::PeanoHilbertKey;
    use crate::domain::segment::Segment;
    use crate::domain::ParticleData;

    fn get_particles() -> Vec<ParticleData> {
        (0..10)
            .chain(30..40)
            .map(|i| ParticleData {
                key: PeanoHilbertKey(i),
                entity: Entity::from_raw(i as u32),
            })
            .collect()
    }

    #[test]
    fn num_contained_particles() {
        let particles = get_particles();
        let check_num = |start: usize, end: usize, num: usize| {
            assert_eq!(
                Segment::new(
                    &particles,
                    PeanoHilbertKey(start as u64),
                    PeanoHilbertKey(end as u64)
                )
                .num_particles,
                num
            );
        };
        check_num(0, 0, 0); // empty
        check_num(0, 1, 1); // contains: 0
        check_num(38, 40, 2); // contains: 38, 39

        check_num(20, 20, 0); // empty
        check_num(20, 25, 0); // empty

        check_num(9, 10, 1); // contains: 9
        check_num(9, 11, 1); // contains: 9

        check_num(25, 32, 2); // contains: 30, 31
    }

    #[test]
    fn get_segments_reaches_desired_size() {
        let particles = get_particles();
        let desired_size = 4;
        let segments = super::get_segments(&particles, desired_size);
        for segment in segments.iter() {
            assert!(segment.num_particles <= desired_size);
        }
    }

    #[test]
    fn get_segments_has_correct_total_number_of_particles() {
        let particles = get_particles();
        let desired_size = 4;
        let segments = super::get_segments(&particles, desired_size);
        assert_eq!(
            segments
                .iter()
                .map(|segment| segment.num_particles)
                .sum::<usize>(),
            particles.len()
        );
    }

    #[test]
    fn get_segments_does_not_fail_with_empty_list() {
        let particles = vec![];
        super::get_segments(&particles, 3);
    }

    #[test]
    fn get_segments_does_not_fail_with_single_particle() {
        let particles = vec![get_particles().remove(0)];
        super::get_segments(&particles, 3);
    }
}
