use crate::app::controllers::{dao_controller, proposal_controller};
use actix_web::web;

pub fn setup_routes(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig {
    cfg.service((
        dao_controller::create,
        dao_controller::find_all_daos,
        dao_controller::find_by_id,
        dao_controller::delete_dao,
        dao_controller::update_dao,
    ))
    .service((
        proposal_controller::create,
        // proposal_controller::get_merkle_proof_from_pub,
        proposal_controller::vote_on_proposal,
        proposal_controller::submit_aggregated_snark,
        proposal_controller::get_proposal,
        proposal_controller::get_proposals,
        proposal_controller::get_all_proposals_by_dao,
        // proposal_controller::send_voter_dto,
        proposal_controller::get_results,
        proposal_controller::get_proposal_by_uid,
    ))
}