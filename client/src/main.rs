use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    Multiaddr, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
};

const RENDEZVOUS_POINT_ADDRESS_STR: &str = "/ip4/0.0.0.0/udp/37080/quic-v1";
const RENDEZVOUS_POINT_PEER_ID_STR: &str = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";
const EXTERNAL_ADDRESS_STR: &str = "/ip4/vpn-se.pujak.ru/udp/37080/quic-v1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rendezvous_point_address = RENDEZVOUS_POINT_ADDRESS_STR.parse::<Multiaddr>()?;
    let rendezvous_point = RENDEZVOUS_POINT_PEER_ID_STR.parse()?;

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| MyBehaviour {
            rendezvous: rendezvous::client::Behaviour::new(key.clone()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        })?
        .build();

    let external_address = EXTERNAL_ADDRESS_STR.parse::<Multiaddr>()?;
    swarm.add_external_address(external_address);

    swarm.dial(rendezvous_point_address.clone())?;

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                cause: Some(error),
                ..
            } if peer_id == rendezvous_point => {
                println!("Lost connection to rendezvous point {}", error);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == rendezvous_point => {
                if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                    rendezvous::Namespace::from_static("rendezvous"),
                    rendezvous_point,
                    None,
                ) {
                    println!("Failed to register: {error}");
                }
                println!("Connection established with rendezvous point {}", peer_id);
            }
            // once `/identify` did its job, we know our external address and can register
            SwarmEvent::Behaviour(MyBehaviourEvent::Rendezvous(
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
            SwarmEvent::Behaviour(MyBehaviourEvent::Rendezvous(
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
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            })) if peer != rendezvous_point => {
                println!("Ping to {} is {}ms", peer, rtt.as_millis())
            }
            other => {
                println!("Unhandled {:?}", other);
            }
        }
    }
    Ok(())
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    rendezvous: rendezvous::client::Behaviour,
    ping: ping::Behaviour,
}
