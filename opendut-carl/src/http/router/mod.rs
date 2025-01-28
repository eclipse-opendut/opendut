use axum::extract::State;
use axum::Json;
use opendut_types::lea::LeaConfig;

pub mod cleo;
pub mod edgar;

pub async fn lea_config(State(config): State<LeaConfig>) -> Json<LeaConfig> {
    Json(Clone::clone(&config))
}