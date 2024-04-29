#![allow(non_snake_case)]
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use proto::market::FileInfoHash;
use serde::{Deserialize, Serialize};

use crate::{
    consumer::encode::{self, encode_user, try_decode_user, EncodedUser},
    producer::{
        self,
        jobs::{JobListItem, JobStatus},
    },
    ServerState,
};

///
/// JOBS ENDPOINTS
///

#[derive(Deserialize)]
struct AddJob {
    fileHash: String,
    peerId: String,
}
async fn add_job(State(state): State<ServerState>, Json(job): Json<AddJob>) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let file_info_hash = FileInfoHash(job.fileHash);
    let peer_id = job.peerId;
    let mut user = None;
    let file_info = match config.get_market_client().await {
        Ok(market) => match market.check_holders(file_info_hash.clone()).await {
            Ok(res) => {
                for producer in res.holders {
                    if peer_id == producer.id {
                        user = Some(producer);
                        break;
                    }
                }
                res.file_info.unwrap()
            }
            _ => return (StatusCode::SERVICE_UNAVAILABLE, "No holders of file").into_response(),
        },
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "Market not available").into_response(),
    };
    let user = match user {
        Some(user) => user,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Holder {peer_id} not found"),
            )
                .into_response()
        }
    };

    let job_id = config
        .jobs_mut()
        .add_job(
            file_info_hash,
            file_info.file_size as u64,
            file_info.file_name,
            user.price,
            peer_id.clone(),
            encode_user(&user),
        )
        .await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(r#"{{"jobID":"{job_id}"}}"#)))
        .unwrap()
}

// returns all peers hosting a given file
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct PeerResponse {
    peers: Vec<Peer>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Peer {
    peerID: String,
    ip: String,
    region: String,
    price: f64,
}
async fn find_peer(
    State(state): State<ServerState>,
    Path(fileHash): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    let response = match config.get_market_client().await {
        Ok(market) => match market.check_holders(FileInfoHash(fileHash)).await {
            Ok(holders) => holders,
            Err(_) => {
                return (StatusCode::SERVICE_UNAVAILABLE, "Could not check holders").into_response()
            }
        },
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Could not connect to market",
            )
                .into_response()
        }
    };
    let peers: Vec<_> = response
        .holders
        .into_iter()
        .map(|user| Peer {
            peerID: user.id,
            ip: user.ip,
            region: "US".into(),
            price: user.price as f64,
        })
        .collect();
    let peers_serialized = serde_json::to_string(&peers).expect("to serialize");

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(r#"{{"peers":{peers_serialized}}}"#)))
        .unwrap()
}

async fn get_job_list(State(state): State<ServerState>) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    let jobs_list = config.jobs_mut().get_jobs_list().await;

    let jobs_json = serde_json::to_string(&jobs_list).expect("to serialize");
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(r#"{{"jobs": {jobs_json}}}"#)))
        .unwrap()
}

async fn get_job_info(
    State(state): State<ServerState>,
    Path(jobID): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let job_info = match config.jobs_mut().get_job_info(&jobID).await {
        Some(job_info) => job_info,
        None => return (StatusCode::NOT_FOUND, "Job not found").into_response(),
    };

    let info_json = serde_json::to_string(&job_info).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(info_json))
        .unwrap()
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct JobPeerInfo {
    ipAddress: String,
    region: String,
    name: String,
    price: i64,
}
async fn job_peer_info(
    State(state): State<ServerState>,
    Path(jobID): Path<String>,
) -> impl IntoResponse {
    let config = state.config.lock().await;

    let user = match config.jobs().get_job(&jobID).await {
        Some(job) => {
            let lock = job.lock().await;
            match try_decode_user(lock.encoded_producer.as_str()) {
                Ok(user) => user,
                Err(_) => return (StatusCode::NOT_FOUND, "Failed to decode user").into_response(),
            }
        }
        None => return (StatusCode::NOT_FOUND, "Job not found").into_response(),
    };

    let peer_info = JobPeerInfo {
        ipAddress: user.ip,
        region: "US".into(),
        name: user.name,
        price: user.price,
    };

    let info_json = serde_json::to_string(&peer_info).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(info_json))
        .unwrap()
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct JobIds {
    jobIDs: Vec<String>,
}
async fn start_jobs(
    State(state): State<ServerState>,
    Json(arg): Json<JobIds>,
) -> impl IntoResponse {
    for job_id in arg.jobIDs {
        let mut config = state.config.lock().await;
        match config.jobs().get_job(&job_id).await {
            Some(job) => {
                let token = config.get_token(job.lock().await.encoded_producer.clone());
                producer::jobs::start(job, token).await;
            }
            None => return (StatusCode::NOT_FOUND, "Job not found").into_response(),
        }
    }
    StatusCode::OK.into_response()
}

async fn pause_jobs(
    State(state): State<ServerState>,
    Json(arg): Json<JobIds>,
) -> impl IntoResponse {
    let mut num_failed = 0;
    for job_id in arg.jobIDs {
        let config = state.config.lock().await;
        match config.jobs().get_job(&job_id).await {
            Some(job) => {
                let mut lock = job.lock().await;
                lock.pause();
            }
            None => num_failed += 1,
        }
    }
    if num_failed > 0 {
        (
            StatusCode::NOT_FOUND,
            format!("Failed to find {num_failed} jobs"),
        )
            .into_response()
    } else {
        (StatusCode::OK, Body::empty()).into_response()
    }
}

async fn terminate_jobs(
    State(state): State<ServerState>,
    Json(arg): Json<JobIds>,
) -> impl IntoResponse {
    let mut num_failed = 0;
    for job_id in arg.jobIDs {
        let config = state.config.lock().await;
        match config.jobs().get_job(&job_id).await {
            Some(job) => {
                let mut lock = job.lock().await;
                lock.terminate();
            }
            None => num_failed += 1,
        }
    }
    if num_failed > 0 {
        (
            StatusCode::NOT_FOUND,
            format!("Failed to find {num_failed} jobs"),
        )
            .into_response()
    } else {
        (StatusCode::OK, Body::empty()).into_response()
    }
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/add-job", put(add_job))
        .route("/find-peer/:fileHash", get(find_peer))
        .route("/job-list", get(get_job_list))
        .route("/job-info/:jobID", get(get_job_info))
        .route("/job-peer/:jobId", get(job_peer_info))
        .route("/start-jobs", put(start_jobs))
        .route("/pause-jobs", put(pause_jobs))
        .route("/terminate-jobs", put(terminate_jobs))
}

/// TESTS

#[allow(dead_code)]
const BASE_URL: &str = "http://localhost:3000";
#[allow(dead_code)]
const GIRAFFE_HASH: &str = "908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227";

#[tokio::test]
#[ignore]
async fn bubbleguppies_test_find_peers() {
    let client = reqwest::Client::new();
    // not registered in market
    let find_res = client
        .get(format!("{BASE_URL}/find-peer/{GIRAFFE_HASH}"))
        .send()
        .await
        .expect("a response");

    // file should not have been added yet
    assert_ne!(find_res.status(), StatusCode::OK);

    //let PeerResponse { peers } = find_res.json().await.expect("to deserialize");
}
