use crate::app::routes::setup_routes;
use actix_web::web;

pub fn initialize(cfg: &mut web::ServiceConfig) {
    setup_routes(cfg);
}