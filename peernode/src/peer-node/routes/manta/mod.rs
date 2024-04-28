pub mod mining;
pub mod stats;
pub mod wallet;

use crate::ServerState;
use axum::Router;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct PubKey {
    pub_key: String,
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .nest("/wallet", mining::routes())
        .merge(stats::routes())
        .merge(wallet::routes())
}
