use axum::extract::State;
use axum::Json;
use crate::http::state::LeaConfig;

pub mod cleo;

pub async fn lea_config(State(config): State<LeaConfig>) -> Json<LeaConfig> {
    Json(Clone::clone(&config))
}