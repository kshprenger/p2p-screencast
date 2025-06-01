use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    Multiaddr, StreamProtocol, Swarm, ping, rendezvous,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
};
use serde::{Deserialize, Serialize};

const RENDEZVOUS_POINT_ADDRESS_STR: &str = "/ip4/94.228.163.43/udp/37080/quic-v1";
const RENDEZVOUS_POINT_PEER_ID_STR: &str = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";
const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(100500);
const RENDEZVOUS_NAMESPACE: &str = "namespace";

fn create_swarm() -> Swarm<Behaviour> {
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
            req_res: request_response::cbor::Behaviour::<BroadcastLockReq, BroadcastLockRes>::new(
                [(StreamProtocol::new("/broadcast/1"), ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
        })
        .unwrap()
        .with_swarm_config(|config| config.with_idle_connection_timeout(IDLE_CONNECTION_TIMEOUT))
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rendezvous_point_address = RENDEZVOUS_POINT_ADDRESS_STR.parse::<Multiaddr>()?;
    let rendezvous_point = RENDEZVOUS_POINT_PEER_ID_STR.parse()?;

    let mut swarm = create_swarm();
    let local_peer_id = swarm.local_peer_id().to_string();

    swarm.add_external_address(rendezvous_point_address.clone());
    swarm.dial(rendezvous_point_address.clone())?;

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::ConnectionClosed {
                peer_id,
                cause: Some(error),
                ..
            } => {
                println!(
                    "Lost connection to rendezvous point {} Reconnecting..",
                    error
                );
                swarm.dial(rendezvous_point_address.clone())?;
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                    rendezvous::Namespace::from_static(RENDEZVOUS_NAMESPACE),
                    rendezvous_point,
                    None,
                ) {
                    println!("Failed to register: {error}");
                }
                println!("Connection established with rendezvous point {}", peer_id);
                swarm
                    .behaviour_mut()
                    .req_res
                    .send_request(&peer_id, BroadcastLockReq {});
            }

            SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                rendezvous::client::Event::Registered {
                    namespace,
                    ttl,
                    rendezvous_node,
                },
            )) => {
                println!(
                    "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                    namespace, rendezvous_node, ttl
                );
            }

            SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                rendezvous::client::Event::RegisterFailed {
                    rendezvous_node,
                    namespace,
                    error,
                },
            )) => {
                println!(
                    "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                    rendezvous_node, namespace, error
                );
            }

            SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                rendezvous::client::Event::Discovered { .. },
            )) => {
                println!("Discovery done");
            }

            SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            })) => {
                println!("Ping to {} is {}ms", peer, rtt.as_millis());
                swarm.behaviour_mut().rendezvous.discover(
                    Some(rendezvous::Namespace::new(RENDEZVOUS_NAMESPACE.to_string()).unwrap()),
                    None,
                    None,
                    rendezvous_point,
                );
            }

            _ => {}
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct BroadcastLockReq {}

#[derive(Debug, Serialize, Deserialize)]
struct BroadcastLockRes {
    res: Result<(), ()>,
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    rendezvous: rendezvous::client::Behaviour,
    ping: ping::Behaviour,
    req_res: request_response::cbor::Behaviour<BroadcastLockReq, BroadcastLockRes>,
}
