use std::sync::Arc;

use tokio::sync::Mutex;
use cmx_infra::database::DatabasePool;
use cmx_utils::config::Config;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub db_pool: DatabasePool,
    pub redis: Mutex<redis::aio::MultiplexedConnection>,
}
