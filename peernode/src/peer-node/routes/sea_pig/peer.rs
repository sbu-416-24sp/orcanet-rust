#![allow(non_snake_case)]

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use orcanet_market::Peer;
use proto::market::User;
use serde::{Deserialize, Serialize};

use crate::ServerState;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct PeerInfo {
    Location: String,
    Latency: String,
    PeerID: String,
    Connection: String,
    OpenStreams: String,
}

async fn get_peer(
    State(state): State<ServerState>,
    Path(peer_id): Path<String>,
) -> impl IntoResponse {
    let config = state.config.lock().await;
    let peer_info = config.get_peer(&peer_id);
    match peer_info {
        Some(peer_info) => {
            let peer_info = PeerInfo {
                Location: "US".into(),
                Latency: "999ms".into(),
                PeerID: peer_id,
                Connection: "connected".into(),
                OpenStreams: "none".into(),
            };

            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(
                    serde_json::to_string(&peer_info).expect("to serialize"),
                ))
                .unwrap()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_peers(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config.lock().await;

    let peers: Vec<_> = config
        .get_peers()
        .into_iter()
        .map(|peer| PeerInfo {
            Location: todo!(),
            Latency: todo!(),
            PeerID: peer.id,
            Connection: todo!(),
            OpenStreams: todo!(),
        })
        .collect();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&peers).expect("to serialize"),
        ))
        .unwrap()
}

async fn remove_peer(
    State(state): State<ServerState>,
    Path(peer_id): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    match config.remove_peer(&peer_id) {
        Some(_) => StatusCode::OK.into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to remove peer").into_response(),
    }
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/get-peer/:peer_id", get(get_peer))
        .route("/get-peers/", get(get_peers))
        .route("/remove-peer/:peer_id", post(remove_peer))
}
