use axum::{
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use tokio::sync::Mutex;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tokio_util::io::ReaderStream;

use crate::producer::db;

#[derive(Clone)]
struct AppState {
    consumers: Arc<Mutex<db::Consumers>>,
}

#[derive(Deserialize, Debug)]
struct FileParams {
    chunk: Option<u64>,
}

#[axum::debug_handler]
async fn handle_file_request(
    params: Path<String>,
    query: Query<FileParams>,
    state: State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    // Obtain file hash, chunk, and consumer address
    let hash = params.0;
    let chunk = query.chunk.unwrap_or(0);
    let address = connect_info.0.ip().to_string();

    // Lock the consumers map
    let mut consumers = state.consumers.lock().await;

    // Parse the Authorization header
    let mut auth_token = if let Some(auth) = headers.get("Authorization") {
        auth.to_str().unwrap_or_default()
    } else {
        ""
    };

    // Remove the "Bearer " prefix
    if !auth_token.is_empty() && auth_token.starts_with("Bearer ") {
        auth_token = &auth_token[7..];
    }

    // Get the consumer
    let consumer = match consumers.get_consumer(&hash) {
        Some(consumer) => consumer,
        None => {
            // Create a new consumer
            let consumer = db::Consumer {
                wallet_address: "wallet_address".to_string(),
                requests: HashMap::new(),
            };

            consumers.add_consumer(hash.clone(), consumer);
            consumers.get_consumer(&hash).unwrap()
        }
    };

    // Get the consumer request
    let request = match consumer.requests.get_mut(&address) {
        Some(request) => request,
        None => {
            // Create a new consumer request
            let request = db::ConsumerRequest {
                chunks_sent: 0,
                access_token: "".to_string(),
            };

            consumer.requests.insert(address.clone(), request);
            consumer.requests.get_mut(&address).unwrap()
        }
    };

    // Check if an access token is expected (first chunk is "free")
    if request.chunks_sent == 0 {
        request.access_token = db::generate_access_token();
    } else if request.access_token != auth_token {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // Send the file chunk (not implemented, send the full file instead)
    let file = match tokio::fs::File::open("giraffe.jpg").await {
        Ok(file) => file,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Increment the chunks sent
    request.chunks_sent += 1;

    println!("Sending file chunk {} for {} to consumer {}", chunk, hash, address);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(
            header::CONTENT_DISPOSITION,
            "inline; filename=\"giraffe.jpg\"",
        )
        .header("X-Access-Token", request.access_token.as_str())
        .body(body)
        .unwrap()
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/file/:file_hash", get(handle_file_request))
        .with_state(AppState {
            consumers: Arc::new(Mutex::new(db::Consumers::new())),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("Listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
