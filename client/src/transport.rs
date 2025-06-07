use libp2p::Swarm;

use crate::behaviour::Behaviour;
use futures::StreamExt;
use tokio::sync::mpsc;

const RENDEZVOUS_NAMESPACE: &str = "namespace";

// Sends/manages network and ui events
pub(super) struct Transport {
    swarm: Swarm<Behaviour>,
    client_requests_rx: mpsc::Receiver<()>,
}

impl Transport {
    pub(super) fn new(swarm: Swarm<Behaviour>, client_requests_rx: mpsc::Receiver<()>) -> Self {
        Transport {
            swarm,
            client_requests_rx,
        }
    }

    pub(super) async fn run_network(&mut self) {
        loop {
            tokio::select! {
                Some(swarm_event) = self.swarm.next() =>{}
                client_event = self.client_requests_rx.recv() =>{}
            }
        }
    }
}

// match event {
//     SwarmEvent::ConnectionClosed {
//         peer_id,
//         cause: Some(error),
//         ..
//     } => {
//         println!(
//             "Lost connection to rendezvous point {} Reconnecting..",
//             error
//         );
//         swarm.dial(rendezvous_point_address.clone())?;
//     }

//     SwarmEvent::ConnectionEstablished { peer_id, .. } => {
//         if let Err(error) = swarm.behaviour_mut().rendezvous.register(
//             rendezvous::Namespace::from_static(RENDEZVOUS_NAMESPACE),
//             rendezvous_point,
//             None,
//         ) {
//             println!("Failed to register: {error}");
//         }
//         println!("Connection established with rendezvous point {}", peer_id);
//         swarm
//             .behaviour_mut()
//             .req_res
//             .send_request(&peer_id, BroadcastLockReq {});
//     }

//     SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
//         rendezvous::client::Event::Registered {
//             namespace,
//             ttl,
//             rendezvous_node,
//         },
//     )) => {
//         println!(
//             "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
//             namespace, rendezvous_node, ttl
//         );
//     }

//     SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
//         rendezvous::client::Event::RegisterFailed {
//             rendezvous_node,
//             namespace,
//             error,
//         },
//     )) => {
//         println!(
//             "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
//             rendezvous_node, namespace, error
//         );
//     }

//     SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
//         rendezvous::client::Event::Discovered { .. },
//     )) => {
//         println!("Discovery done");
//     }

//     SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event {
//         peer,
//         result: Ok(rtt),
//         ..
//     })) => {
//         println!("Ping to {} is {}ms", peer, rtt.as_millis());
//         swarm.behaviour_mut().rendezvous.discover(
//             Some(rendezvous::Namespace::new(RENDEZVOUS_NAMESPACE.to_string()).unwrap()),
//             None,
//             None,
//             rendezvous_point,
//         );
//     }

//     _ => {}
