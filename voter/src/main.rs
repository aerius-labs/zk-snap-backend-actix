use actix_web::{App, HttpServer};

pub mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(app::init::initialize))
        .bind("127.0.0.1:9090")?
        .run()
        .await
}