use libp2p::{ping, rendezvous, request_response, swarm::NetworkBehaviour};
use serde::{Deserialize, Serialize};

// Lock screen broadcast with PeerId
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BroadcastLockReq {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BroadcastLockRes {
    res: Result<(), ()>,
}

// Stream from peer that acquired broadcast lock
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StreamReq {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StreamRes {}

#[derive(NetworkBehaviour)]
pub(crate) struct Behaviour {
    pub(crate) rendezvous: rendezvous::client::Behaviour,
    pub(crate) ping: ping::Behaviour,
    pub(crate) lock: request_response::cbor::Behaviour<BroadcastLockReq, BroadcastLockRes>,
    pub(crate) stream: request_response::cbor::Behaviour<StreamReq, StreamRes>,
}
