mod mpi_types;
#[cfg(all(test, not(feature = "mpi")))]
mod tests;

use self::mpi_types::IntoEquivalenceType;
use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::SearchData;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::domain::QuadTree;
use crate::domain::TopLevelIndices;
use crate::parameters::SimulationBox;
use crate::prelude::WorldRank;
use crate::quadtree::radius_search::bounding_boxes_overlap_periodic;
use crate::units::Length;
use crate::units::MVec;
use crate::units::VecLength;
use crate::voronoi::utils::Extent;
use crate::voronoi::ActiveDimension;
use crate::voronoi::Dimension;
use crate::voronoi::Point;

type MpiSearchData<D> = <SearchData<D> as IntoEquivalenceType>::Equiv;
type MpiSearchResult<D> = <SearchResult<D> as IntoEquivalenceType>::Equiv;

pub struct ParallelSearch<'a, D: Dimension + 'static>
where
    SearchData<D>: IntoEquivalenceType,
    SearchResult<D>: IntoEquivalenceType,
{
    data_comm: &'a mut ExchangeCommunicator<MpiSearchData<D>>,
    result_comm: &'a mut ExchangeCommunicator<MpiSearchResult<D>>,
    global_extent: Extent<Point<D>>,
    tree: &'a QuadTree,
    indices: &'a TopLevelIndices,
    box_: SimulationBox,
    rank: WorldRank,
}

type OutgoingRequests<D> = DataByRank<Vec<MpiSearchData<D>>>;
type IncomingRequests<D> = DataByRank<Vec<SearchData<D>>>;
type OutgoingResults<D> = DataByRank<Vec<MpiSearchResult<D>>>;

impl<'a> ParallelSearch<'a, ActiveDimension> {
    fn tree_node_and_search_overlap(
        &self,
        tree: &QuadTree,
        search: &SearchData<ActiveDimension>,
    ) -> bool {
        bounding_boxes_overlap_periodic(
            &self.box_,
            &tree.extent.center,
            &tree.extent.side_lengths(),
            &VecLength::new_unchecked(search.point),
            &VecLength::from_vector_and_scale(MVec::ONE, Length::new_unchecked(search.radius)),
        )
    }

    fn get_requests_by_rank(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> OutgoingRequests<ActiveDimension> {
        let mut outgoing = DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.data_comm);
        for (rank, indices_this_rank) in self.indices.iter() {
            if *rank == *self.rank {
                continue;
            }
            for i in indices_this_rank.iter() {
                let subtree = &self.tree[i];
                for search in data.iter() {
                    if self.tree_node_and_search_overlap(subtree, &search) {
                        outgoing[*rank].push(search.to_equivalent());
                    }
                }
            }
        }
        outgoing
    }

    fn exchange_all(
        &mut self,
        outgoing: OutgoingRequests<ActiveDimension>,
    ) -> IncomingRequests<ActiveDimension> {
        let mut incoming = self.data_comm.exchange_all(outgoing);
        incoming
            .drain_all()
            .map(|(rank, requests)| {
                (
                    rank,
                    requests
                        .into_iter()
                        .map(|request| SearchData::<ActiveDimension>::from_equivalent(&request))
                        .collect(),
                )
            })
            .collect()
    }

    fn get_search_result(
        &self,
        incoming: IncomingRequests<ActiveDimension>,
    ) -> OutgoingResults<ActiveDimension> {
        let mut outgoing =
            DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.result_comm);
        for (rank, data) in incoming.iter() {
            for search in data {
                let particles = self.tree.get_particles_in_radius(
                    &self.box_,
                    &VecLength::new_unchecked(search.point),
                    &Length::new_unchecked(search.radius),
                );
                outgoing[*rank].extend(particles.into_iter().map(|p| {
                    SearchResult::from_search(&search, p.pos.value_unchecked()).to_equivalent()
                }));
            }
        }
        outgoing
    }
}

impl<'a> RadiusSearch<ActiveDimension> for ParallelSearch<'a, ActiveDimension> {
    fn unique_radius_search(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> Vec<SearchResult<ActiveDimension>> {
        let outgoing = self.get_requests_by_rank(data);
        let incoming = self.exchange_all(outgoing);
        let search_result = self.get_search_result(incoming);
        vec![]
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<ActiveDimension>>> {
        Some(self.global_extent.clone())
    }
}
