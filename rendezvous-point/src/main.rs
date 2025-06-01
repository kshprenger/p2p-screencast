use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    PeerId, Swarm, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
};

const CONNECTION_ADDR: &str = "/ip4/0.0.0.0/udp/37080/quic-v1";
const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(100500);

#[derive(NetworkBehaviour)]
struct Behaviour {
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}

fn create_swarm() -> Swarm<Behaviour> {
    libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| Behaviour {
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: ping::Behaviour::new(
                ping::Config::new()
                    .with_interval(PING_INTERVAL)
                    .with_timeout(PING_TIMEOUT),
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

    let mut swarm = create_swarm();

    let _ = swarm.listen_on(CONNECTION_ADDR.parse()?)?;
    println!("Start listening on {}", CONNECTION_ADDR);

    while let Some(event) = swarm.next().await {
        match event {
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
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            other => {
                println!("Unhandled {:?}", other);
            }
        }
    }

    Ok(())
}
