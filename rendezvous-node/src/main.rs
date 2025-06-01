use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    PeerId, StreamProtocol, Swarm,
    identity::Keypair,
    ping, rendezvous,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
};
use serde::{Deserialize, Serialize};

const CONNECTION_ADDR: &str = "/ip4/0.0.0.0/udp/37080/quic-v1";
const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(100500);

#[derive(Debug, Serialize, Deserialize)]
struct BroadcastLockReq {}

#[derive(Debug, Serialize, Deserialize)]
struct BroadcastLockRes {
    res: Result<(), ()>,
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
    req_res: request_response::cbor::Behaviour<BroadcastLockReq, BroadcastLockRes>,
}

fn create_swarm(keypair: Keypair) -> Swarm<Behaviour> {
    libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| Behaviour {
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
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
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32])?;
    println!("PeerId: {}", PeerId::from(keypair.public()));

    let mut swarm = create_swarm(keypair);

    let _ = swarm.listen_on(CONNECTION_ADDR.parse()?)?;
    println!("Start listening on {}", CONNECTION_ADDR);

    let mut lock: String = String::new();

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {}", peer_id);
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Disconnected from {}", peer_id);
            }

            SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                rendezvous::server::Event::PeerRegistered { peer, registration },
            )) => {
                println!(
                    "Peer {} registered for namespace '{}'",
                    peer, registration.namespace
                );
            }

            SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                rendezvous::server::Event::DiscoverServed {
                    enquirer,
                    registrations,
                },
            )) => {
                println!(
                    "Served peer {} with {} registrations",
                    enquirer,
                    registrations.len()
                );
            }

            SwarmEvent::Behaviour(BehaviourEvent::ReqRes(request_response::Event::Message {
                peer,
                message,
            })) => {
                if lock.to_string() == "" {
                    lock = peer.to_string();
                    println!("Locked on {}", lock)
                } else {
                    println!("Already locked")
                }
            }

            _ => {}
        }
    }

    Ok(())
}
