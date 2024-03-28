use crate::app::aggregator_controller::{generate_base_proof, generate_recursive_proof};
use crate::app::dtos::MessageType;
use futures::StreamExt;
use lapin::{options::*, Consumer};
use std::thread;
use std::time::Duration;

pub async fn consume_messages(mut consumer: Consumer) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(result) = consumer.next().await {
        let delivery = result?;
        let delivery_data = delivery.data.clone();
        let message: MessageType = serde_json::from_slice(&delivery_data)?;

        match message {
            MessageType::Base(base_dto) => {
                log::info!("Received struct from queue: {:?}", base_dto);
                match generate_base_proof(base_dto).await {
                    Ok(_) => log::info!("Base proof generated successfully"),
                    Err(e) => log::info!("Error generating base proof: {:?}", e),
                };
            }
            MessageType::Recursive(reccursive_dto) => {
                log::info!("Received reccursive dto from queue");
                match generate_recursive_proof(reccursive_dto).await {
                    Ok(_) => log::info!("Recursive proof generated successfully"),
                    Err(e) => log::info!("Error generating recursive proof: {:?}", e),
                };
                
            }
        }

        // Delay for 10 seconds
        thread::sleep(Duration::from_secs(10));

        // Acknowledge the message
        let _ = delivery.ack(BasicAckOptions::default()).await;
    }

    Ok(())
}
