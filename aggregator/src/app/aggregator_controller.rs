use crate::app::dtos::{AggregatorBaseDto, AggregatorRecursiveDto, VoterDto};
use actix_web::{post, web, HttpResponse, Responder};
use aggregator::wrapper::common::Snark;
use serde_json::json;
use std::io::{Error, ErrorKind};
use std::env;


pub async fn generate_base_proof(input: AggregatorBaseDto) -> Result<(), Error> {
   
    let result = super::aggregator_service::generate_base_proof(input).await?;
    let len = result.instances[0].len();
    log::info!("{:?}", result.instances[0][len-2]);

    // Submit calculated base proof to proposal db
    submit_snark(result).await?;

    Ok(())
}

async fn submit_snark(proof: Snark) -> Result<(), Error> {
    // Submit calculated snark to proposal db
    let url = env::var("BACKEND_ADDR").unwrap_or_else(|_| "http://localhost:8080/proposal/agg/".to_string());
    let client = reqwest::Client::new();
    let res = client.post(url)
        .json(&proof)
        .send()
        .await;

    match res {
        Ok(_) => {
            log::info!("Snark submitted successfully");
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    Ok(())
}

pub async fn generate_recursive_proof(input: AggregatorRecursiveDto) -> Result<(), Error>{
    let result = match super::aggregator_service::generate_recursive_proof(input).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    // Submit calculated recursive proof to proposal db
    submit_snark(result).await?;
    Ok(())
}

// ! Only for testing
// TODO: Remove after testing
#[post("vote/")]
async fn generate_vote_proof(dto: web::Json<VoterDto>) -> impl Responder {
    let input = dto.into_inner();

    let result = match super::aggregator_service::generate_voter_proof(input).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to generate vote proof",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(result)
}
