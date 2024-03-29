use actix_web::web;
use crate::app::voter_controller;

pub fn setup_routes(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig {
    cfg.service((
        voter_controller::generate_vote_proof,
    ))
}