use mpi::traits::Equivalence;
use raxiom::communication::DataByRank;
use raxiom::communication::ExchangeCommunicator;
use raxiom::communication::Rank;
use raxiom::communication::SizedCommunicator;
use raxiom::hash_map::HashMap;
use raxiom::hash_map::HashSet;
use raxiom::prelude::ParticleId;

use super::UniqueParticleId;

#[derive(Equivalence, Clone, Debug, PartialEq, Eq, Hash)]
struct IdLookupRequest {
    id: UniqueParticleId,
}

#[derive(Equivalence)]
struct IdLookupReply {
    request_id: UniqueParticleId,
    id: ParticleId,
}

pub struct IdCache {
    map: HashMap<UniqueParticleId, ParticleId>,
    rank: Rank,
    requests: HashSet<IdLookupRequest>,
}

impl IdCache {
    pub fn new(map: HashMap<UniqueParticleId, ParticleId>, rank: Rank) -> Self {
        IdCache {
            map,
            rank,
            requests: HashSet::default(),
        }
    }

    pub fn lookup(&mut self, id: UniqueParticleId) -> Option<ParticleId> {
        self.map.get(&id).copied()
    }

    pub fn is_local(&self, id: UniqueParticleId) -> bool {
        self.map
            .get(&id)
            .map(|id| id.rank == self.rank)
            .unwrap_or(false)
    }

    pub fn perform_lookup(&mut self) {
        let mut request_comm: ExchangeCommunicator<IdLookupRequest> = ExchangeCommunicator::new();
        let mut reply_comm: ExchangeCommunicator<IdLookupReply> = ExchangeCommunicator::new();
        // For now: ask everyone everything
        let mut outgoing_requests = DataByRank::empty();
        let requests: Vec<_> = self.requests.drain().collect();
        for rank in request_comm.other_ranks() {
            outgoing_requests.insert(rank, requests.clone());
        }
        let incoming_requests = request_comm.exchange_all(outgoing_requests);
        let mut outgoing_replies = DataByRank::empty();
        for (rank, incoming_requests) in incoming_requests.iter() {
            let outgoing_replies_this_rank: Vec<_> = incoming_requests
                .into_iter()
                .filter_map(|incoming_request| {
                    self.lookup(incoming_request.id).map(|id| IdLookupReply {
                        request_id: incoming_request.id,
                        id,
                    })
                })
                .collect();
            outgoing_replies.insert(rank, outgoing_replies_this_rank);
        }
        let incoming_replies = reply_comm.exchange_all(outgoing_replies);
        for (_, incoming_replies) in incoming_replies {
            self.map.extend(
                incoming_replies
                    .into_iter()
                    .map(|reply| (reply.request_id, reply.id)),
            );
        }
    }

    pub fn add_lookup_request_if_necessary(&mut self, id: UniqueParticleId) {
        if !self.is_local(id) {
            self.requests.insert(IdLookupRequest { id });
        }
    }
}
