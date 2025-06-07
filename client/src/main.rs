use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    Multiaddr, StreamProtocol, Swarm, ping, rendezvous,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
};
use serde::{Deserialize, Serialize};
use swarm::create_swarm;

mod behaviour;
mod swarm;
mod transport;
mod ui;

const RENDEZVOUS_POINT_ADDRESS_STR: &str = "/ip4/94.228.163.43/udp/37080/quic-v1";
const MESSAGE_BOUND: usize = 100;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rendezvous_point_address = RENDEZVOUS_POINT_ADDRESS_STR.parse::<Multiaddr>()?;

    let mut swarm = create_swarm();
    swarm.add_external_address(rendezvous_point_address.clone());
    swarm.dial(rendezvous_point_address.clone())?;

    let (client_tx, client_rx) = tokio::sync::mpsc::channel::<()>(MESSAGE_BOUND);

    let mut transport = transport::Transport::new(swarm, client_rx);

    tokio::spawn(async move {
        transport.run_network().await;
    });

    // Go(ui)

    Ok(())
}
