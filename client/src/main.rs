use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use url::Url;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;

mod coordinator;
mod transport;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create a MediaEngine to manage media codecs
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs().unwrap();

    // Create the API object
    let api = APIBuilder::new().with_media_engine(media_engine).build();

    let config = RTCConfiguration {
        ice_servers: vec![webrtc::ice_transport::ice_server::RTCIceServer {
            urls: vec!["stun:stun1.l.google.com:5349".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a PeerConnectionFactory
    let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());
    // Set up signaling (e.g., WebSocket) here...

    // Set up ICE Candidate handling
    setup_ice_candidates(peer_connection.clone()).await;

    let _data_channel = peer_connection
        .create_data_channel("test", None)
        .await
        .unwrap();
    // Create an offer to start ICE candidate gathering
    let offer = peer_connection.create_offer(None).await.unwrap();
    peer_connection.set_local_description(offer).await.unwrap();

    // Set up track handling
    setup_tracks(peer_connection.clone()).await;

    println!("WebRTC application has started successfully.");
    Ok(())
}

async fn setup_ice_candidates(peer_connection: Arc<RTCPeerConnection>) {
    peer_connection.on_ice_candidate(Box::new(|candidate: Option<RTCIceCandidate>| {
        println!("New ICE candidate: {:?}", candidate);

        if let Some(candidate) = candidate {
            println!("New ICE candidate: {:?}", candidate);
            // Send the candidate to the remote peer through signaling server
        }
        Box::pin(async {})
    }));
}

async fn setup_tracks(peer_connection: Arc<RTCPeerConnection>) {
    peer_connection.on_track(Box::new(|track, _| {
        println!("New track received: {:?}", track);
        Box::pin(async {})
    }));
}
