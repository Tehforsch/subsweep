use mpi::ffi::MPI_Request;
use mpi::request::scope;
use mpi::request::Request;
use mpi::request::Scope;

use super::task::FluxData;
use crate::communication::DataByRank;
use crate::communication::DataCommunicator;
use crate::communication::Rank;

type OutstandingRequest = MPI_Request;

pub struct SweepCommunicator {
    communicator: DataCommunicator<FluxData>,
    send_buffers: DataByRank<Vec<FluxData>>,
    requests: DataByRank<Option<OutstandingRequest>>,
}

impl SweepCommunicator {
    pub fn new(communicator: DataCommunicator<FluxData>) -> Self {
        let send_buffers = DataByRank::from_communicator(&communicator);
        let requests = DataByRank::from_communicator(&communicator);
        Self {
            communicator,
            send_buffers,
            requests,
        }
    }

    pub fn try_send_all(&mut self, to_send: &mut DataByRank<Vec<FluxData>>) {
        for (rank, data) in to_send.iter_mut() {
            if let Some(request) = self.requests[*rank] {
                self.request_completed(*rank, request);
            } else {
                self.send_buffers[*rank].extend(data.drain(..));
                self.requests[*rank] = scope(|scope| {
                    let scoped_request = self.communicator.immediate_send_vec(
                        scope,
                        *rank,
                        &self.send_buffers[*rank][..],
                    );
                    // SAFETY:
                    // We only overwrite the data in a send buffer whenever the previous request is finished.
                    // We also await all requests before dropping the send buffers.
                    unsafe { scoped_request.map(|scoped_request| scoped_request.into_raw().0) }
                })
            }
        }
    }

    fn request_completed(&self, rank: Rank, request: MPI_Request) {
        scope(|s| {
            let data = &self.send_buffers[rank];
            match self.to_scoped_request(s, &data, rank, request).test() {
                Ok(_status) => true,
                Err(_) => false,
            }
        });
    }

    fn wait_for_request(&self, rank: Rank, request: MPI_Request) {
        scope(|s| {
            let data = &self.send_buffers[rank];
            self.to_scoped_request(s, &data, rank, request).wait();
        });
    }

    fn to_scoped_request<'a, Sc: Scope<'a>>(
        &self,
        scope: Sc,
        data: &'a Vec<FluxData>,
        rank: Rank,
        request: MPI_Request,
    ) -> Request<'a, [FluxData], Sc> {
        unsafe { Request::from_raw(request, &data, scope) }
    }
}

// Make sure we cannot accidentally drop the send buffers while
// there are still pending MPI requests.
impl Drop for SweepCommunicator {
    fn drop(&mut self) {
        for (rank, request) in self.requests.iter() {
            if let Some(request) = request {
                self.wait_for_request(*rank, *request);
            }
        }
    }
}
