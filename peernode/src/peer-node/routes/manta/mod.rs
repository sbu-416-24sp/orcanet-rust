pub mod mining;
pub mod stats;
pub mod wallet;

use axum::Router;
use crate::ServerState;

pub fn routes() -> Router<ServerState> {
    Router::new()
}
