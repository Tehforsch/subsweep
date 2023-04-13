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
use super::halo_cache::HaloCache;
use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::halo_iteration::SearchResults;
use super::SearchData;
use crate::communication::communicator::Communicator;
use crate::communication::exchange_communicator::ExchangeCommunicator;
use crate::communication::DataByRank;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::dimension::ActiveDimension;
use crate::dimension::Point;
use crate::domain::Decomposition;
use crate::domain::QuadTree;
use crate::extent::Extent;
use crate::parameters::SimulationBox;
use crate::quadtree::LeafDataType;
use crate::simulation_box::PeriodicWrapType3d;
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
    finished_comm: &'a mut Communicator<SendNum>,
    global_extent: Extent<Point<D>>,
    tree: &'a QuadTree,
    decomposition: &'a Decomposition,
    box_: SimulationBox,
    halo_cache: HaloCache,
}

type OutgoingRequests<D> = DataByRank<Vec<MpiSearchData<D>>>;
type IncomingRequests<D> = DataByRank<Vec<SearchData<D>>>;
type OutgoingResults<D> = DataByRank<Vec<MpiSearchResult<D>>>;
type IncomingResults<D> = DataByRank<SearchResults<D>>;

fn find_wrapped_point(
    box_: &SimulationBox,
    search: &SearchData<ActiveDimension>,
    point: VecLength,
) -> (PeriodicWrapType3d, Point<ActiveDimension>) {
    let mut iter = box_
        .iter_periodic_images(point)
        .map(|(t, p)| (t, p.value_unchecked()))
        .filter(|(_, p)| search.point.distance(*p) < search.radius);
    let result = iter.next().unwrap();
    assert!(
        iter.next().is_none(),
        "Search radius large enough that two periodic images fall into it at the same time."
    );
    result
}

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

        for (rank, data) in incoming.iter() {
            for search in data {
                let result = self.get_haloes_from_search(*rank, search);
                new_haloes[*rank].extend(result.map(|x| x.to_equivalent()));
            }
        }
        new_haloes
    }

    fn get_local_periodic_haloes(
        &mut self,
        data: &Vec<SearchData<ActiveDimension>>,
    ) -> Vec<SearchResult<ActiveDimension>> {
        let mut new_haloes = vec![];
        for search in data.iter() {
            let haloes = self.get_haloes_from_search(self.data_comm.rank(), search);
            new_haloes.extend(
                haloes
                    .into_iter()
                    .filter(|h| h.periodic_wrap_type.is_periodic()),
            );
        }
        new_haloes
    }

    fn get_haloes_from_search<'b>(
        &'b mut self,
        rank: Rank,
        search: &'b SearchData<ActiveDimension>,
    ) -> impl Iterator<Item = SearchResult<ActiveDimension>> + 'b {
        let pos = VecLength::new_unchecked(search.point);
        let radius = Length::new_unchecked(search.radius);
        let particles = self.tree.iter_particles_in_radius(&self.box_, pos, radius);
        self.halo_cache.get_new_haloes::<ActiveDimension>(
            rank,
            particles.into_iter().map(|p| {
                let (wrap_type, wrapped_p) = find_wrapped_point(&self.box_, search, *p.pos());
                (wrapped_p, p.id, wrap_type)
            }),
        )
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
        let mut incoming_new_haloes = self.result_comm.exchange_all(outgoing);
        incoming_new_haloes
            .drain_all()
            .map(|(rank, results)| {
                (
                    rank,
                    results
                        .into_iter()
                        .map(|request| SearchResult::<ActiveDimension>::from_equivalent(&request))
                        .collect(),
                )
            })
            .collect()
    }

    fn print_num_new_haloes(&mut self, num_new_haloes: usize) {
        let num_new_haloes: SendNum = self.finished_comm.all_gather_sum(&SendNum(num_new_haloes));
        debug!("{} new haloes imported.", num_new_haloes.0);
    }

    fn rank(&self) -> Rank {
        self.data_comm.rank()
    }
}

impl<'a> RadiusSearch<ActiveDimension> for ParallelSearch<'a, ActiveDimension> {
    fn radius_search(
        &mut self,
        data: Vec<SearchData<ActiveDimension>>,
    ) -> DataByRank<SearchResults<ActiveDimension>> {
        let local_periodic_haloes = self.get_local_periodic_haloes(&data);
        let outgoing = self.get_outgoing_searches(data);
        let incoming = self.exchange_all_searches(outgoing);
        let outgoing_results = self.get_outgoing_results(incoming);
        let mut incoming_results = self.exchange_all_results(outgoing_results);
        incoming_results.insert(self.rank(), local_periodic_haloes);
        self.print_num_new_haloes(incoming_results.size());
        incoming_results
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

    fn rank(&self) -> Rank {
        self.data_comm.rank()
    }
}
