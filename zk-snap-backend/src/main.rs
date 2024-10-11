use crate::app::config::init_mongo;
use crate::app::entities::{dao_entity::Dao, proposal_entity::Proposal};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use app::repository::generic_repository::Repository;

pub mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    // let db_client = web::Data::new(init_mongo().await);
    let client = match init_mongo().await {
        Ok(client) => client,
        Err(e) => panic!("Failed to connect to MongoDB: {}", e),
    };
    let database = client.database("rust");
    let dao_collection = database.collection::<Dao>("dao");
    let proposal_collection = database.collection::<Proposal>("proposal");

    let dao_service = Repository::new(dao_collection);
    let dao_service_data = web::Data::new(dao_service);

    let proposal_service = Repository::new(proposal_collection);
    let proposal_service_data = web::Data::new(proposal_service);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
            ])
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(dao_service_data.clone())
            .app_data(proposal_service_data.clone())
            .configure(app::init::initialize)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}