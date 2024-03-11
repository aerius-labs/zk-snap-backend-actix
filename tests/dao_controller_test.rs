use actix_web::{http::StatusCode, test, web, App};
use zk_snap_backend_actix::app::controllers::dao_controller::{create, delete_dao, find_by_id};
use zk_snap_backend_actix::app::dtos::dao_dto::CreateDaoDto;
use zk_snap_backend_actix::app::entities::dao_entity::Dao;
use zk_snap_backend_actix::app::repository::repository::Repository;

#[cfg(test)]
mod tests {
    use mongodb::Client;

    use super::*;

    #[actix_web::test]
    // #[ignore = "requires mongodb"]
    async fn test_create_dao() {
        let uri = "mongodb://localhost:27017";
        let client = Client::with_uri_str(uri)
            .await
            .expect("Failed to connect to MongoDB");

        let database = client.database("test");
        let collection = database.collection::<Dao>("daos");
        let repository = Repository::new(collection);
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(repository))
                .service(create),
        )
        .await;

        let dao = CreateDaoDto {
            name: "".to_string(),
            description: "DAO description".to_string(),
            logo: Some("DAO logo".to_string()),
            members: vec!["member1".to_string(), "member2".to_string()],
        };

        let req = test::TestRequest::post()
            .uri("/dao/")
            .set_json(&dao)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = test::read_body(resp).await;
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "Invalid input");
    }

    #[actix_web::test]
    async fn test_get_dao() {
        let uri = "mongodb://localhost:27017";
        let client = Client::with_uri_str(uri)
            .await
            .expect("Failed to connect to MongoDB");

        let database = client.database("test");
        let collection = database.collection::<Dao>("daos");
        let repository = Repository::new(collection);
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(repository))
                .service(create)
                .service(find_by_id)
                .service(delete_dao),
        )
        .await;

        let dao = CreateDaoDto {
            name: "test Dao".to_string(),
            description: "DAO description".to_string(),
            logo: Some("DAO logo".to_string()),
            members: vec!["member1".to_string(), "member2".to_string()],
        };

        let post_req = test::TestRequest::post()
            .uri("/dao/")
            .set_json(&dao)
            .to_request();

        let post_resp = test::call_service(&mut app, post_req).await;
        assert_eq!(post_resp.status(), StatusCode::CREATED);
        let body = test::read_body(post_resp).await;
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = body["ObjectId"]["$oid"].as_str().unwrap();

        let get_req = test::TestRequest::get()
            .uri(&format!("/dao/{}", id))
            .to_request();
        let get_resp = test::call_service(&mut app, get_req).await;
        assert_eq!(get_resp.status(), StatusCode::OK);
        let body = test::read_body(get_resp).await;
        let body: Dao = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.name, "test Dao");

        let delete_req = test::TestRequest::delete()
            .uri(&format!("/dao/{}", id))
            .to_request();
        let delete_resp = test::call_service(&mut app, delete_req).await;
        assert_eq!(delete_resp.status(), StatusCode::OK);
        let body = test::read_body(delete_resp).await;
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "Deleted DAO");
    }
}
