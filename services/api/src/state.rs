use std::sync::Arc;
use sqlx::mysql::MySqlPool;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<MySqlPool>,
    pub config: Arc<Config>,
}
