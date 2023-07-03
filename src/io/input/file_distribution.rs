use crate::communication::Rank;

#[derive(Debug)]
struct Position {
    file_index: usize,
    pos: usize,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub file_index: usize,
    pub start: usize,
    pub end: usize,
}

impl Region {
    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug)]
pub struct RankAssignment {
    pub regions: Vec<Region>,
}

fn get_regions_from(
    num_entries_per_file: &[usize],
    mut start: Position,
    num_entries: usize,
) -> (Vec<Region>, Position) {
    let mut regions = vec![];
    if start.pos == num_entries_per_file[start.file_index] {
        start.file_index += 1;
        start.pos = 0;
    }
    let mut end = Position {
        file_index: start.file_index,
        pos: start.pos + num_entries,
    };
    while num_entries_per_file[end.file_index] < end.pos {
        regions.push(Region {
            file_index: end.file_index,
            start: start.pos,
            end: num_entries_per_file[end.file_index],
        });
        end.pos -= num_entries_per_file[end.file_index];
        end.file_index += 1;
        start.pos = 0;
        start.file_index += 1;
    }
    regions.push(Region {
        file_index: end.file_index,
        start: start.pos,
        end: end.pos,
    });
    (regions, end)
}

pub fn get_rank_assignment_for_rank(
    num_entries_per_file: &[usize],
    num_ranks: usize,
    rank: Rank,
) -> RankAssignment {
    get_rank_assignment(num_entries_per_file, num_ranks).remove(rank as usize)
}

pub fn get_rank_assignment(
    num_entries_per_file: &[usize],
    num_ranks: usize,
) -> Vec<RankAssignment> {
    let num_entries: usize = num_entries_per_file.iter().sum();
    let num_entries_per_rank = num_entries / num_ranks;
    let num_entries_last_rank = num_entries - num_entries_per_rank * (num_ranks - 1);
    let mut start = Position {
        file_index: 0,
        pos: 0,
    };
    let mut assignments = vec![];
    for rank in 0..num_ranks {
        let num_entries = if rank == num_ranks - 1 {
            num_entries_last_rank
        } else {
            num_entries_per_rank
        };
        let (regions, end) = get_regions_from(num_entries_per_file, start, num_entries);
        start = end;
        let regions = regions
            .into_iter()
            .filter(|region| region.size() > 0)
            .collect();
        assignments.push(RankAssignment { regions });
    }
    assignments
}

#[cfg(test)]
mod tests {
    use crate::io::input::file_distribution::get_rank_assignment;

    #[test]
    fn rank_assignment() {
        let assignment = get_rank_assignment(&[100], 1);
        assert_eq!(assignment.len(), 1);
        assert_eq!(assignment[0].regions.len(), 1);
        assert_eq!(assignment[0].regions[0].file_index, 0);
        assert_eq!(assignment[0].regions[0].start, 0);
        assert_eq!(assignment[0].regions[0].end, 100);
        let assignment = get_rank_assignment(&[100], 2);
        assert_eq!(assignment.len(), 2);
        assert_eq!(assignment[0].regions.len(), 1);
        assert_eq!(assignment[0].regions[0].file_index, 0);
        assert_eq!(assignment[0].regions[0].start, 0);
        assert_eq!(assignment[0].regions[0].end, 50);
        assert_eq!(assignment[1].regions.len(), 1);
        assert_eq!(assignment[1].regions[0].file_index, 0);
        assert_eq!(assignment[1].regions[0].start, 50);
        assert_eq!(assignment[1].regions[0].end, 100);
        let assignment = get_rank_assignment(&[100], 3);
        assert_eq!(assignment.len(), 3);
        assert_eq!(assignment[0].regions.len(), 1);
        assert_eq!(assignment[0].regions[0].file_index, 0);
        assert_eq!(assignment[0].regions[0].start, 0);
        assert_eq!(assignment[0].regions[0].end, 33);
        assert_eq!(assignment[1].regions.len(), 1);
        assert_eq!(assignment[1].regions[0].file_index, 0);
        assert_eq!(assignment[1].regions[0].start, 33);
        assert_eq!(assignment[1].regions[0].end, 66);
        assert_eq!(assignment[2].regions.len(), 1);
        assert_eq!(assignment[2].regions[0].file_index, 0);
        assert_eq!(assignment[2].regions[0].start, 66);
        assert_eq!(assignment[2].regions[0].end, 100);
        let assignment = get_rank_assignment(&[100, 200], 3);
        assert_eq!(assignment.len(), 3);
        assert_eq!(assignment[0].regions.len(), 1);
        assert_eq!(assignment[0].regions[0].file_index, 0);
        assert_eq!(assignment[0].regions[0].start, 0);
        assert_eq!(assignment[0].regions[0].end, 100);
        assert_eq!(assignment[1].regions.len(), 1);
        assert_eq!(assignment[1].regions[0].file_index, 1);
        assert_eq!(assignment[1].regions[0].start, 0);
        assert_eq!(assignment[1].regions[0].end, 100);
        assert_eq!(assignment[2].regions.len(), 1);
        assert_eq!(assignment[2].regions[0].file_index, 1);
        assert_eq!(assignment[2].regions[0].start, 100);
        assert_eq!(assignment[2].regions[0].end, 200);
        let assignment = get_rank_assignment(&[100, 200, 301], 4);
        assert_eq!(assignment.len(), 4);
        assert_eq!(assignment[0].regions.len(), 2);
        assert_eq!(assignment[0].regions[0].file_index, 0);
        assert_eq!(assignment[0].regions[0].start, 0);
        assert_eq!(assignment[0].regions[0].end, 100);
        assert_eq!(assignment[0].regions[1].file_index, 1);
        assert_eq!(assignment[0].regions[1].start, 0);
        assert_eq!(assignment[0].regions[1].end, 50);
        assert_eq!(assignment[1].regions.len(), 1);
        assert_eq!(assignment[1].regions[0].file_index, 1);
        assert_eq!(assignment[1].regions[0].start, 50);
        assert_eq!(assignment[1].regions[0].end, 200);
        assert_eq!(assignment[2].regions.len(), 1);
        assert_eq!(assignment[2].regions[0].file_index, 2);
        assert_eq!(assignment[2].regions[0].start, 0);
        assert_eq!(assignment[2].regions[0].end, 150);
        assert_eq!(assignment[3].regions.len(), 1);
        assert_eq!(assignment[3].regions[0].file_index, 2);
        assert_eq!(assignment[3].regions[0].start, 150);
        assert_eq!(assignment[3].regions[0].end, 301);
        let assignment = get_rank_assignment(&[100, 0, 100], 2);
        assert_eq!(assignment.len(), 2);
        assert_eq!(assignment[0].regions.len(), 1);
        assert_eq!(assignment[0].regions[0].file_index, 0);
        assert_eq!(assignment[0].regions[0].start, 0);
        assert_eq!(assignment[0].regions[0].end, 100);
        assert_eq!(assignment[1].regions.len(), 1);
        assert_eq!(assignment[1].regions[0].file_index, 2);
        assert_eq!(assignment[1].regions[0].start, 0);
        assert_eq!(assignment[1].regions[0].end, 100);
    }
}
