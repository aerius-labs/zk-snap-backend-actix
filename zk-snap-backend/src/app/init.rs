use crate::app::routes::setup_routes;
use actix_web::web;

/// Initialize the application
pub fn initialize(cfg: &mut web::ServiceConfig) {
    setup_routes(cfg);
}
