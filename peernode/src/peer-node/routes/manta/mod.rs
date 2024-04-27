pub mod mining;
pub mod stats;
pub mod wallet;

use crate::ServerState;
use axum::Router;

pub fn routes() -> Router<ServerState> {
    Router::new()
}
