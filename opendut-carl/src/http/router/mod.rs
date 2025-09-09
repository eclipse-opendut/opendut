use axum::extract::State;
use axum::Json;
use opendut_model::lea::LeaConfig;

pub mod cleo;
pub mod edgar;
pub mod arch;

pub async fn lea_config(State(config): State<LeaConfig>) -> Json<LeaConfig> {
    Json(Clone::clone(&config))
}
