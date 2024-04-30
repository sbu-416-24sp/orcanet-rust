use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::{Number, Value};

use crate::ServerState;

use super::PubKey;
#[derive(Debug, Serialize)]
struct NetworkStats {
    _id: String,
    pub_key: String,
    incoming_speed: String,
    outgoing_speed: String,
}

async fn network(
    State(state): State<ServerState>,
    Json(PubKey { pub_key }): Json<PubKey>,
) -> impl IntoResponse {
    let res = NetworkStats {
        _id: "65680d250505420b42427a82".into(),
        pub_key,
        incoming_speed: "30".into(),
        outgoing_speed: "2000".into(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&res).expect("to serialize"),
        ))
        .unwrap()
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct FileTypes {
    _id: String,
    pub_key: String,
    filetype: Vec<String>,
    filetypeNumber: Vec<serde_json::Value>,
}

async fn file_types(
    State(state): State<ServerState>,
    Json(PubKey { pub_key }): Json<PubKey>,
) -> impl IntoResponse {
    let mut counts = serde_json::Map::new();
    let filetypes = vec!["mp4", "mp4", "exe"];
    for t in filetypes {
        let n = match counts.get(t) {
            Some(&Value::Number(ref n)) => n.as_i64().unwrap_or(0),
            _ => 0,
        };
        counts.insert(t.to_string(), Value::Number(Number::from(n + 1)));
    }

    let filetype = counts.keys().map(|k| k.clone()).collect();

    let file_types = FileTypes {
        _id: "65680d250505420b42427a82".into(),
        pub_key,
        filetype,
        filetypeNumber: vec![Value::Object(counts)], // yes
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&file_types).expect("to serialize"),
        ))
        .unwrap()
}

#[derive(Debug, Serialize)]
struct ActivityResponse {
    _id: String,
    pub_key: String,
    activities: Vec<ActivityDay>,
}
#[derive(Debug, Serialize)]
struct ActivityDay {
    date: String,
    download: String,
    upload: String,
}

async fn activity(
    State(state): State<ServerState>,
    Json(PubKey { pub_key }): Json<PubKey>,
) -> impl IntoResponse {
    let res = ActivityResponse {
        _id: "65680d250505420b42427a82".into(),
        pub_key,
        activities: vec![ActivityDay {
            date: "3/30".into(),
            download: "10".into(),
            upload: "2".into(),
        }],
    };
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&res).expect("to serialize"),
        ))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/network", get(network))
        .route("/types", get(file_types))
        .route("/activity", get(activity))
}
