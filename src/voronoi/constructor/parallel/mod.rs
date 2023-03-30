mod mpi_types;
mod plugin;
#[cfg(all(test, not(feature = "mpi")))]
mod tests;

use bevy::prelude::info;
use bevy::prelude::Entity;
use derive_more::Add;
use derive_more::Sum;
use mpi::traits::Equivalence;
pub use plugin::ParallelVoronoiGridConstruction;

use self::mpi_types::IntoEquivalenceType;
pub use super::halo_iteration::HaloExporter;
use super::halo_iteration::IndexedRadiusSearch;
use super::halo_iteration::IndexedSearchResult;
use super::halo_iteration::SearchResult;
use super::SearchData;
use crate::communication::communicator::Communicator;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::communication::SizedCommunicator;
use crate::domain::QuadTree;
use crate::domain::TopLevelIndices;
use crate::mpidbg;
use crate::parameters::SimulationBox;
use crate::quadtree::radius_search::bounding_boxes_overlap_periodic;
use crate::units::Length;
use crate::units::MVec;
use crate::units::VecLength;
use crate::voronoi::utils::Extent;
use crate::voronoi::ActiveDimension;
use crate::voronoi::Dimension;
use crate::voronoi::Point;

type MpiSearchData<D> = <SearchData<D> as IntoEquivalenceType>::Equiv;
type MpiSearchResult<D> = <IndexedSearchResult<D, Entity> as IntoEquivalenceType>::Equiv;

#[derive(Clone, Add, Sum, Equivalence)]
pub struct NumUndecided(pub usize);

pub struct ParallelSearch<'a, D: Dimension + 'static>
where
    SearchData<D>: IntoEquivalenceType,
    IndexedSearchResult<D, Entity>: IntoEquivalenceType,
{
    data_comm: &'a mut ExchangeCommunicator<MpiSearchData<D>>,
    result_comm: &'a mut ExchangeCommunicator<MpiSearchResult<D>>,
    finished_comm: &'a mut Communicator<NumUndecided>,
    global_extent: Extent<Point<D>>,
    tree: &'a QuadTree,

    indices: &'a TopLevelIndices,
    box_: SimulationBox,
}

type OutgoingRequests<D> = DataByRank<Vec<MpiSearchData<D>>>;
type IncomingRequests<D> = DataByRank<Vec<SearchData<D>>>;
type OutgoingResults<D> = DataByRank<Vec<MpiSearchResult<D>>>;
type IncomingResults<D> = DataByRank<Vec<IndexedSearchResult<D, Entity>>>;

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

    fn get_outgoing_searches(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> OutgoingRequests<ActiveDimension> {
        let mut outgoing = DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.data_comm);
        let rank_owns_part_of_search_radius = |rank, search| {
            self.indices[rank].iter().any(|index| {
                let subtree = &self.tree[index];
                self.tree_node_and_search_overlap(subtree, search)
            })
        };
        for rank in self.data_comm.other_ranks() {
            for search in data.iter() {
                if rank_owns_part_of_search_radius(rank, search) {
                    outgoing[rank].push(search.to_equivalent());
                }
            }
        }
        outgoing
    }

    fn get_outgoing_results(
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
                    let result = SearchResult::from_search(search, p.pos.value_unchecked());
                    let indexed_result = IndexedSearchResult {
                        result,
                        point_index: p.entity,
                    };
                    indexed_result.to_equivalent()
                }));
            }
        }
        outgoing
    }

    fn exchange_all_searches(
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

    fn exchange_all_results(
        &mut self,
        outgoing: OutgoingResults<ActiveDimension>,
    ) -> IncomingResults<ActiveDimension> {
        let mut incoming = self.result_comm.exchange_all(outgoing);
        incoming
            .drain_all()
            .map(|(rank, requests)| {
                (
                    rank,
                    requests
                        .into_iter()
                        .map(|request| {
                            IndexedSearchResult::<ActiveDimension, Entity>::from_equivalent(
                                &request,
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }
}

impl<'a> IndexedRadiusSearch<ActiveDimension> for ParallelSearch<'a, ActiveDimension> {
    type Index = Entity;

    fn radius_search(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> DataByRank<Vec<IndexedSearchResult<ActiveDimension, Entity>>> {
        let outgoing = self.get_outgoing_searches(data);
        let incoming = self.exchange_all_searches(outgoing);
        let outgoing_results = self.get_outgoing_results(incoming);
        self.exchange_all_results(outgoing_results)
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<ActiveDimension>>> {
        Some(self.global_extent.clone())
    }

    fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool {
        let total_undecided: NumUndecided = self
            .finished_comm
            .all_gather_sum(&NumUndecided(num_undecided_this_rank));
        info!("{} tetras undecided", total_undecided.0);
        total_undecided.0 == 0
    }
}
