use std::sync::Arc;

use cmx_utils::config;
// use ::redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;

use cmx_infra::{database::Database, redis};
use crate::{
    api::server,
    application::{state::AppState},
    
};

pub async fn run() {
    // Load configuration.
    let config = config::load();

    // Connect to Redis.
    let redis = redis::open(&config).await;

    // Connect to PostgreSQL.
    let db_pool = Database::connect(config.clone().into())
        .await
        .expect("Failed to connect to the database.");

    // Run migrations.
    Database::migrate(&db_pool)
        .await
        .expect("Failed to run database migrations.");

    // Build the application state.
    let shared_state = Arc::new(AppState {
        config,
        db_pool,
        redis: Mutex::new(redis),
    });

    server::start(shared_state).await;
}
