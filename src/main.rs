use actix_web::{web, App, HttpServer};
use crate::app::config::init_mongo;

pub mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let db_client = web::Data::new(init_mongo().await);

    HttpServer::new(move || {
        App::new()
            .app_data(db_client.clone())
            .configure(app::init::initialize)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}