mod mpi_types;

use self::mpi_types::IntoEquivalenceType;
use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::SearchData;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::domain::QuadTree;
use crate::domain::TopLevelIndices;
use crate::voronoi::utils::Extent;
use crate::voronoi::Dimension;
use crate::voronoi::Point;

type MpiSearchData<D> = <SearchData<D> as IntoEquivalenceType>::Equiv;

pub struct ParallelSearch<'a, D: Dimension + 'static>
where
    SearchData<D>: IntoEquivalenceType,
{
    data_comm: &'a mut ExchangeCommunicator<MpiSearchData<D>>,
    result_comm: &'a mut ExchangeCommunicator<SearchResult<D>>,
    global_extent: Extent<Point<D>>,
    tree: &'a QuadTree,
    indices: &'a TopLevelIndices,
}

fn tree_node_and_search_overlap<D: Dimension>(tree: &QuadTree, search: &SearchData<D>) -> bool {
    todo!()
}

type OutgoingRequests<D> = DataByRank<Vec<MpiSearchData<D>>>;
type IncomingRequests<D> = DataByRank<Vec<SearchData<D>>>;

impl<'a, D> ParallelSearch<'a, D>
where
    D: Clone,
    D: Dimension,
    SearchData<D>: IntoEquivalenceType,
    MpiSearchData<D>: Clone,
{
    fn get_requests_by_rank(&mut self, data: Vec<SearchData<D>>) -> OutgoingRequests<D> {
        let mut outgoing = DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.data_comm);
        for (rank, indices_this_rank) in self.indices.iter() {
            for i in indices_this_rank.iter() {
                let subtree = &self.tree[i];
                for search in data.iter() {
                    if tree_node_and_search_overlap(subtree, &search) {
                        outgoing[*rank].push(search.to_equivalent());
                    }
                }
            }
        }
        outgoing
    }

    fn exchange_all(&mut self, outgoing: OutgoingRequests<D>) -> IncomingRequests<D> {
        let mut incoming = self.data_comm.exchange_all(outgoing);
        incoming
            .drain_all()
            .map(|(rank, requests)| {
                (
                    rank,
                    requests
                        .into_iter()
                        .map(|request| SearchData::<D>::from_equivalent(&request))
                        .collect(),
                )
            })
            .collect()
    }
}

impl<'a, D> RadiusSearch<D> for ParallelSearch<'a, D>
where
    D: Clone,
    D: Dimension,
    SearchData<D>: IntoEquivalenceType,
    MpiSearchData<D>: Clone,
{
    fn unique_radius_search(&mut self, data: Vec<SearchData<D>>) -> Vec<SearchResult<D>> {
        let outgoing_requests_by_rank = self.get_requests_by_rank(data);
        todo!()
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
        Some(self.global_extent.clone())
    }
}
