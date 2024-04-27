pub mod market;
pub mod settings;

use axum::Router;
use crate::ServerState;

pub fn routes() -> Router<ServerState> {
    Router::new()
}
