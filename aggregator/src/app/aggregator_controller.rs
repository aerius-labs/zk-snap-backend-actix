use crate::app::dtos::{AggregatorBaseDto, AggregatorRecursiveDto, ProofFromAggregator};
use std::io::{Error, ErrorKind};
use std::env;


pub async fn generate_base_proof(input: AggregatorBaseDto) -> Result<(), Error> {
   
    let result = super::aggregator_service::generate_base_proof(input).await?;
    let len = result.instances[0].len();
    log::info!("{:?}", result.instances[0][len-2]);

    // Submit calculated base proof to proposal db
    let res = ProofFromAggregator {
        proof: result,
        is_base: true,
    };
    submit_snark(res).await?;

    Ok(())
}

async fn submit_snark(proof: ProofFromAggregator) -> Result<(), Error> {
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
            Ok(())
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }
}

pub async fn generate_recursive_proof(input: AggregatorRecursiveDto) -> Result<(), Error>{
    let result = match super::aggregator_service::generate_recursive_proof(input).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    // Submit calculated recursive proof to proposal db
    let res = ProofFromAggregator {
        proof: result,
        is_base: false,
    };
    submit_snark(res).await?;
    Ok(())
}

