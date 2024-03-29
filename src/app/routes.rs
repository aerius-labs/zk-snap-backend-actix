use crate::app::controllers::{dao_controller, proposal_controller};
use actix_web::web;

pub fn setup_routes(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig {
    cfg.service((
        dao_controller::create,
        dao_controller::find_all_daos,
        dao_controller::find_by_id,
        dao_controller::delete_dao,
        dao_controller::update_dao,
        proposal_controller::create,
        proposal_controller::get_merkle_proof_from_pub,
        proposal_controller::vote_on_proposal,
        proposal_controller::submit_aggregated_snark,
        proposal_controller::get_proposal,
    ))
}
