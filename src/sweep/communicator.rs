use mpi::request::scope;
use mpi::request::Request;

use super::task::FluxData;
use crate::communication::DataByRank;
use crate::communication::DataCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;

#[cfg(feature = "mpi")]
type OutstandingRequest = mpi::ffi::MPI_Request;

#[cfg(not(feature = "mpi"))]
type OutstandingRequest = ();

pub struct SweepCommunicator<'comm> {
    communicator: &'comm mut DataCommunicator<FluxData>,
    send_buffers: DataByRank<Vec<FluxData>>,
    requests: DataByRank<Option<OutstandingRequest>>,
}

#[cfg(feature = "mpi")]
fn to_unscoped<'a>(
    scoped_request: Request<'a, [FluxData], &mpi::request::LocalScope<'a>>,
) -> OutstandingRequest {
    // SAFETY:
    // We only overwrite the data in a send buffer whenever the previous request is finished.
    // We also await all requests before dropping the send buffers.
    unsafe { scoped_request.into_raw().0 }
}

#[cfg(not(feature = "mpi"))]
fn to_unscoped<'a>(_scoped_request: Request<'a, [FluxData], &mpi::request::LocalScope<'a>>) -> () {
    ()
}

impl<'comm> SweepCommunicator<'comm> {
    pub fn new(communicator: &'comm mut DataCommunicator<FluxData>) -> Self {
        let send_buffers = DataByRank::from_communicator(communicator);
        let requests = DataByRank::from_communicator(communicator);
        Self {
            communicator,
            send_buffers,
            requests,
        }
    }

    pub fn count_remaining_to_send(&self) -> usize {
        self.send_buffers
            .iter()
            .map(|(_, buffer)| buffer.len())
            .sum()
    }

    pub fn update_pending_requests(&mut self) {
        for rank in self.communicator.other_ranks() {
            if self.requests[rank]
                .map(|request| self.request_completed(rank, request))
                .unwrap_or(true)
            {
                self.requests[rank] = None;
                self.send_buffers[rank].clear();
            }
        }
    }

    pub fn try_send_all(&mut self, to_send: &mut DataByRank<Vec<FluxData>>) {
        self.update_pending_requests();
        for (rank, data) in to_send.iter_mut() {
            if data.is_empty() {
                continue;
            }
            if self.requests[*rank].is_none() {
                self.send_buffers[*rank].append(data);
                self.requests[*rank] = scope(|scope| {
                    let scoped_request = self.communicator.immediate_send_vec(
                        scope,
                        *rank,
                        &self.send_buffers[*rank][..],
                    );
                    scoped_request.map(to_unscoped)
                })
            }
        }
    }

    pub fn try_recv(&mut self, rank: Rank) -> Vec<FluxData> {
        self.communicator.receive_vec(rank)
    }

    #[cfg(feature = "mpi")]
    fn request_completed(&self, rank: Rank, request: OutstandingRequest) -> bool {
        scope(|s| {
            let data = &self.send_buffers[rank];
            match self.to_scoped_request(s, data, request).test() {
                Ok(_status) => true,
                Err(_) => false,
            }
        })
    }

    #[cfg(not(feature = "mpi"))]
    fn request_completed(&self, _rank: Rank, _request: OutstandingRequest) -> bool {
        true
    }

    #[cfg(feature = "mpi")]
    fn wait_for_request(&self, rank: Rank, request: OutstandingRequest) {
        scope(|s| {
            let data = &self.send_buffers[rank];
            self.to_scoped_request(s, data, request).wait();
        });
    }

    #[cfg(not(feature = "mpi"))]
    fn wait_for_request(&self, _rank: Rank, _request: OutstandingRequest) {}

    #[cfg(feature = "mpi")]
    fn to_scoped_request<'a, Sc: mpi::request::Scope<'a>>(
        &self,
        scope: Sc,
        data: &'a Vec<FluxData>,
        request: OutstandingRequest,
    ) -> Request<'a, [FluxData], Sc> {
        unsafe { Request::from_raw(request, data, scope) }
    }
}

// Make sure we cannot accidentally drop the send buffers while
// there are still pending MPI requests.
impl<'comm> Drop for SweepCommunicator<'comm> {
    fn drop(&mut self) {
        for (rank, request) in self.requests.iter() {
            if let Some(request) = request {
                self.wait_for_request(*rank, *request);
            }
        }
    }
}

impl<'comm> SizedCommunicator for SweepCommunicator<'comm> {
    fn size(&self) -> usize {
        self.communicator.size()
    }

    fn rank(&self) -> Rank {
        self.communicator.rank()
    }
}
