use crate::app::aggregator_controller;
use actix_web::web;

pub fn setup_routes(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig {
    cfg.service((
        aggregator_controller::generate_base_proof,
        aggregator_controller::generate_recursive_proof,
    ))
}
