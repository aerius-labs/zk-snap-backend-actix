use actix_web::web;
use crate::app::controllers::dao_controller;

pub fn setup_routes(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig{
    cfg.service((dao_controller::create, dao_controller::hello))
}