use crate::app::controllers::dao_controller;
use actix_web::web;

pub fn setup_routes(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig {
    cfg.service((
        dao_controller::create,
        dao_controller::find_all_daos,
        dao_controller::find_by_id,
    ))
}
