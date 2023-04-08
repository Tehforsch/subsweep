mod mpi_types;
mod plugin;
#[cfg(all(test, not(feature = "mpi")))]
mod tests;

use bevy::prelude::debug;
use bevy::prelude::info;
use derive_more::Add;
use derive_more::Sum;
use mpi::traits::Equivalence;
pub use plugin::ParallelVoronoiGridConstruction;

use self::mpi_types::IntoEquivalenceType;
use self::mpi_types::TetraIndexSend;
use super::halo_cache::CachedSearchResult;
use super::halo_cache::HaloCache;
use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::halo_iteration::SearchResults;
use super::SearchData;
use crate::communication::communicator::Communicator;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::communication::SizedCommunicator;
use crate::dimension::ActiveDimension;
use crate::dimension::Point;
use crate::domain::Decomposition;
use crate::domain::QuadTree;
use crate::extent::Extent;
use crate::parameters::SimulationBox;
use crate::units::Length;
use crate::units::VecLength;
use crate::voronoi::DDimension;

type MpiSearchData<D> = <SearchData<D> as IntoEquivalenceType>::Equiv;
type MpiSearchResult<D> = <SearchResult<D> as IntoEquivalenceType>::Equiv;

#[derive(Clone, Add, Sum, Equivalence)]
struct SendNum(pub usize);

pub struct ParallelSearch<'a, D: DDimension + 'static>
where
    SearchData<D>: IntoEquivalenceType,
    SearchResult<D>: IntoEquivalenceType,
{
    data_comm: &'a mut ExchangeCommunicator<MpiSearchData<D>>,
    result_comm: &'a mut ExchangeCommunicator<MpiSearchResult<D>>,
    tetra_index_comm: &'a mut ExchangeCommunicator<TetraIndexSend>,
    finished_comm: &'a mut Communicator<SendNum>,
    global_extent: Extent<Point<D>>,
    tree: &'a QuadTree,
    decomposition: &'a Decomposition,
    box_: SimulationBox,
    halo_cache: HaloCache,
}

type OutgoingRequests<D> = DataByRank<Vec<MpiSearchData<D>>>;
type IncomingRequests<D> = DataByRank<Vec<SearchData<D>>>;
type OutgoingResults<D> = (
    DataByRank<Vec<MpiSearchResult<D>>>,
    DataByRank<Vec<TetraIndexSend>>,
);
type IncomingResults<D> = DataByRank<SearchResults<D>>;

impl<'a> ParallelSearch<'a, ActiveDimension> {
    fn get_outgoing_searches(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> OutgoingRequests<ActiveDimension> {
        let mut outgoing = DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.data_comm);
        for rank in self.data_comm.other_ranks() {
            for search in data.iter() {
                let extent = Extent::<Point<ActiveDimension>>::cube_around_sphere(
                    search.point,
                    search.radius,
                );
                if self.decomposition.rank_owns_part_of_search_radius(
                    rank,
                    &extent,
                    &self.global_extent,
                ) {
                    outgoing[rank].push(search.to_equivalent());
                }
            }
        }
        outgoing
    }

    fn get_outgoing_results(
        &mut self,
        incoming: IncomingRequests<ActiveDimension>,
    ) -> OutgoingResults<ActiveDimension> {
        let mut new_haloes =
            DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.result_comm);
        let mut undecided_tetras =
            DataByRank::same_for_all_ranks_in_communicator(vec![], &*self.result_comm);

        for (rank, data) in incoming.iter() {
            for search in data {
                let particles = self.tree.get_particles_in_radius(
                    &self.box_,
                    &VecLength::new_unchecked(search.point),
                    &Length::new_unchecked(search.radius),
                );
                let result = self.halo_cache.get_closest_new::<ActiveDimension>(
                    *rank,
                    search.point,
                    particles
                        .into_iter()
                        .map(|p| (p.pos.value_unchecked(), p.id)),
                );
                match result {
                    CachedSearchResult::NothingNew => {}
                    CachedSearchResult::NewPoint(result) => {
                        new_haloes[*rank].push(result.to_equivalent());
                        undecided_tetras[*rank].push(search.tetra_index.into());
                    }
                    CachedSearchResult::NewPointThatHasJustBeenExported => {
                        undecided_tetras[*rank].push(search.tetra_index.into());
                    }
                }
            }
        }
        self.halo_cache.flush();
        (new_haloes, undecided_tetras)
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
        let mut incoming_new_haloes = self.result_comm.exchange_all(outgoing.0);
        let mut incoming_undecided_tetras = self.tetra_index_comm.exchange_all(outgoing.1);
        incoming_new_haloes
            .drain_all()
            .map(|(rank, results)| {
                let undecided_tetras = incoming_undecided_tetras
                    .remove(&rank)
                    .unwrap()
                    .into_iter()
                    .map(|t| t.into())
                    .collect();
                (
                    rank,
                    SearchResults {
                        new_haloes: results
                            .into_iter()
                            .map(|request| {
                                SearchResult::<ActiveDimension>::from_equivalent(&request)
                            })
                            .collect(),
                        undecided_tetras,
                    },
                )
            })
            .collect()
    }

    fn print_num_new_haloes(&mut self, num_new_haloes: usize) {
        let num_new_haloes: SendNum = self.finished_comm.all_gather_sum(&SendNum(num_new_haloes));
        debug!("{} new haloes imported.", num_new_haloes.0);
    }
}

impl<'a> RadiusSearch<ActiveDimension> for ParallelSearch<'a, ActiveDimension> {
    fn radius_search(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> DataByRank<SearchResults<ActiveDimension>> {
        let outgoing = self.get_outgoing_searches(data);
        let incoming = self.exchange_all_searches(outgoing);
        let outgoing_results = self.get_outgoing_results(incoming);
        self.print_num_new_haloes(outgoing_results.0.size());
        self.exchange_all_results(outgoing_results)
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<ActiveDimension>>> {
        Some(self.global_extent.clone())
    }

    fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool {
        let total_undecided: SendNum = self
            .finished_comm
            .all_gather_sum(&SendNum(num_undecided_this_rank));
        info!("{} tetras undecided", total_undecided.0);
        total_undecided.0 == 0
    }
}
