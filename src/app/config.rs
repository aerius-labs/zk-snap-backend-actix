use mongodb::{Client, options::ClientOptions};
use std::env;
use dotenv::dotenv;

pub async fn init_mongo() -> mongodb::error::Result<Client> {
    dotenv().ok();
    let mongodb_uri = env::var("MONGO_URI").expect("MONGO_URI must be set");
    let client_options = ClientOptions::parse(mongodb_uri).await.unwrap();
    Client::with_options(client_options)
}