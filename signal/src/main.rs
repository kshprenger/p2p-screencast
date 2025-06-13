use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::Filter;
use warp::ws::Message;

const PORT: u16 = 37080;

#[derive(Debug, Clone)]
struct Peer {
    sender: Option<mpsc::UnboundedSender<Message>>,
}

#[derive(Debug, Clone)]
struct Room {
    peers: HashMap<String, Peer>,
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;

#[derive(Debug, Serialize, Deserialize)]
struct SignalMessage {
    #[serde(rename = "type")]
    message_type: String,
    data: String,
    target: Option<String>,
    room_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JoinResponse {
    peer_id: String,
    other_peers: Vec<String>,
}

fn generate_peer_id() -> String {
    Uuid::new_v4().to_string()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["Content-Type"]);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_rooms(rooms.clone()))
        .map(|ws: warp::ws::Ws, rooms| {
            ws.on_upgrade(move |socket| handle_connection(socket, rooms))
        });

    let routes = ws_route.with(cors).with(warp::log("signaling_server"));

    tracing::info!("Starting signaling server on {} port...", PORT);
    warp::serve(routes).run(([0, 0, 0, 0], PORT)).await;
}

fn with_rooms(
    rooms: Rooms,
) -> impl Filter<Extract = (Rooms,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rooms.clone())
}

async fn handle_connection(ws: warp::ws::WebSocket, rooms: Rooms) {
    use futures::{SinkExt, StreamExt};

    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (sender, receiver) = mpsc::unbounded_channel();
    let mut receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);

    tokio::task::spawn(async move {
        while let Some(message) = receiver.next().await {
            if ws_sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let mut peer_id = None;
    let mut room_id = None;

    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("websocket error: {}", e);
                break;
            }
        };

        if let Ok(text) = msg.to_str() {
            if let Ok(signal) = serde_json::from_str::<SignalMessage>(text) {
                match signal.message_type.as_str() {
                    "join" => {
                        if let Some(join_room_id) = signal.room_id {
                            let new_peer_id = generate_peer_id();
                            peer_id = Some(new_peer_id.clone());
                            room_id = Some(join_room_id.clone());

                            let join_response = handle_join(
                                join_room_id,
                                new_peer_id,
                                rooms.clone(),
                                sender.clone(),
                            )
                            .await;
                            if let Ok(response) = serde_json::to_string(&join_response) {
                                let _ = sender.send(Message::text(response));
                            }
                        }
                    }
                    "leave" => {
                        if let (Some(r_id), Some(p_id)) = (&room_id, &peer_id) {
                            handle_leave(r_id.clone(), p_id.clone(), rooms.clone()).await;
                            break;
                        }
                    }
                    _ => {
                        if let (Some(r_id), Some(p_id)) = (&room_id, &peer_id) {
                            handle_signal(r_id.clone(), p_id.clone(), signal, rooms.clone()).await;
                        }
                    }
                }
            }
        }
    }

    if let (Some(r_id), Some(p_id)) = (room_id, peer_id) {
        handle_leave(r_id, p_id, rooms).await;
    }
}

async fn handle_join(
    room_id: String,
    peer_id: String,
    rooms: Rooms,
    sender: mpsc::UnboundedSender<Message>,
) -> JoinResponse {
    let mut rooms = rooms.lock().unwrap();

    let room = rooms.entry(room_id.clone()).or_insert_with(|| Room {
        peers: HashMap::new(),
    });

    let other_peers: Vec<String> = room.peers.keys().cloned().collect();

    for (_, peer) in room.peers.iter_mut() {
        if let Some(peer_sender) = &peer.sender {
            let notification = SignalMessage {
                message_type: "new-peer".to_string(),
                data: peer_id.clone(),
                target: None,
                room_id: Some(room_id.clone()),
            };
            if let Ok(text) = serde_json::to_string(&notification) {
                let _ = peer_sender.send(Message::text(text));
            }
        }
    }

    room.peers.insert(
        peer_id.clone(),
        Peer {
            sender: Some(sender),
        },
    );

    tracing::info!("New peer {} joined room {}", peer_id, room_id);

    JoinResponse {
        peer_id,
        other_peers,
    }
}

async fn handle_leave(room_id: String, peer_id: String, rooms: Rooms) {
    let mut rooms = rooms.lock().unwrap();

    if let Some(room) = rooms.get_mut(&room_id) {
        room.peers.remove(&peer_id);

        tracing::info!("Peer {} left room {}", peer_id, room_id);

        for (_, peer) in room.peers.iter_mut() {
            if let Some(peer_sender) = &peer.sender {
                let notification = SignalMessage {
                    message_type: "peer-left".to_string(),
                    data: peer_id.clone(),
                    target: None,
                    room_id: Some(room_id.clone()),
                };
                if let Ok(text) = serde_json::to_string(&notification) {
                    let _ = peer_sender.send(Message::text(text));
                }
            }
        }

        if room.peers.is_empty() {
            tracing::info!("Room {} was destroyed", room_id);
            rooms.remove(&room_id);
        }
    }
}

async fn handle_signal(room_id: String, peer_id: String, message: SignalMessage, rooms: Rooms) {
    let mut rooms = rooms.lock().unwrap();

    if let Some(room) = rooms.get_mut(&room_id) {
        if let Some(target_peer_id) = message.target {
            if let Some(target_peer) = room.peers.get(&target_peer_id) {
                if let Some(peer_sender) = &target_peer.sender {
                    let forwarded_message = SignalMessage {
                        message_type: message.message_type,
                        data: message.data,
                        target: Some(peer_id),
                        room_id: Some(room_id.clone()),
                    };
                    tracing::info!(
                        "Prepared signal message {:#?} to other peers",
                        forwarded_message
                    );
                    if let Ok(text) = serde_json::to_string(&forwarded_message) {
                        let _ = peer_sender.send(Message::text(text));
                    }
                }
            }
        }
    }
}
