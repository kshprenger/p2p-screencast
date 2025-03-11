use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    PeerId, identify, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
};

const CONNECTION_ADDR: &str = "/ip4/0.0.0.0/udp/37080/quic-v1";

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32])?;
    println!("PeerId: {}", PeerId::from(keypair.public()));

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| MyBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "rendezvous-example/1.0.0".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        })?
        .build();

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
            SwarmEvent::Behaviour(MyBehaviourEvent::Rendezvous(
                rendezvous::server::Event::PeerRegistered { peer, registration },
            )) => {
                println!(
                    "Peer {} registered for namespace '{}'",
                    peer, registration.namespace
                );
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Rendezvous(
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
