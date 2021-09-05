use std::sync::Arc;
use crate::config::TorrustConfig;
use crate::data::Database;
use crate::auth::AuthorizationService;

pub type Username = String;

pub type WebAppData = actix_web::web::Data<Arc<AppData>>;

pub struct AppData {
    pub cfg: Arc<TorrustConfig>,
    pub database: Arc<Database>,
    pub auth: Arc<AuthorizationService>,
}

impl AppData {
    pub fn new(cfg: Arc<TorrustConfig>, database: Arc<Database>, auth: Arc<AuthorizationService>) -> AppData {
        AppData {
            cfg,
            database,
            auth
        }
    }
}
