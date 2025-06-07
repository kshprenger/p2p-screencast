use std::time::Duration;

use libp2p::{
    StreamProtocol, Swarm, ping, rendezvous,
    request_response::{self, ProtocolSupport},
};

use crate::behaviour::{Behaviour, BroadcastLockReq, BroadcastLockRes, StreamReq, StreamRes};

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(100500);

pub(super) fn create_swarm() -> Swarm<Behaviour> {
    libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| Behaviour {
            rendezvous: rendezvous::client::Behaviour::new(key.clone()),
            ping: ping::Behaviour::new(
                ping::Config::new()
                    .with_interval(PING_INTERVAL)
                    .with_timeout(PING_TIMEOUT),
            ),
            lock: request_response::cbor::Behaviour::<BroadcastLockReq, BroadcastLockRes>::new(
                [(StreamProtocol::new("/broadcast"), ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
            stream: request_response::cbor::Behaviour::<StreamReq, StreamRes>::new(
                [(StreamProtocol::new("/stream"), ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
        })
        .unwrap()
        .with_swarm_config(|config| config.with_idle_connection_timeout(IDLE_CONNECTION_TIMEOUT))
        .build()
}
