use crate::app::config::init_mongo;
use crate::app::entities::dao_entity::Dao;
use actix_web::{web, App, HttpServer};
use app::repository::generic_repository::Repository;

pub mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    // let db_client = web::Data::new(init_mongo().await);
    let client = match init_mongo().await {
        Ok(client) => client,
        Err(e) => panic!("Failed to connect to MongoDB: {}", e),
    };
    let database = client.database("rust");
    let collection = database.collection::<Dao>("dao");

    let dao_service = Repository::new(collection);
    let dao_service_data = web::Data::new(dao_service);

    HttpServer::new(move || {
        App::new()
            .app_data(dao_service_data.clone())
            .configure(app::init::initialize)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
