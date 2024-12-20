use crate::app::repository::generic_repository::Repository;
use crate::app::services::dao_service::{
    create_dao, delete_by_id, get_all_daos, get_dao_by_id, update_dao_by_id,
};
use crate::app::{dtos::dao_dto::CreateDaoDto, entities::dao_entity::Dao};
use actix_web::web::Path;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;
use validator::Validate;

/// Create a new Dao
/// 
/// This endpoint create a new community DAO
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// POST /dao
/// Content-Type: application/json
/// ```
/// 
/// # Request Body
/// 
/// The request must include a JSON body with the following fields:
/// 
/// ```json
/// {
///    "name": "DAO Name",
///    "description": "DAO Description",
///    "logo": "https://www.example.com/logo.png"
/// }
/// ```
/// 
/// # Validation Rules
/// 
/// - `name`: String between 3 and 50 characters
/// - `description`: String between 3 and 200 characters
/// - `logo`: Optional URL string with minimum length of 3 characters
/// 
/// # Response
/// 
/// ## Success (201 Created)
/// 
/// ```json
/// {
///     "message": "Creating DAO",
///     "ObjectId": "507f1f77bcf86cd799439011"
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// Returned when validation fails:
/// ```json
/// {
///     "message": "Invalid input",
///     "errors": {
///         "name": ["Name must be between 3 and 50 characters"]
///     }
/// }
/// ```
/// 
/// Returned when creation fails:
/// ```json
/// {
///     "message": "Failed to create DAO",
///     "error": "Database error message"
/// }
/// ```
/// 
/// # Example Usage
/// 
/// ```bash
/// curl -X POST http://api.example.com/dao \
///      -H "Content-Type: application/json" \
///      -d '{
///           "name": "Example DAO",
///           "description": "A description of the DAO",
///           "logo": "https://example.com/logo.png"
///          }'
/// ```
#[post("/dao")]
async fn create(db: web::Data<Repository<Dao>>, dao: web::Json<CreateDaoDto>) -> impl Responder {
    let dao = dao.into_inner();

    if dao.validate().is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Invalid input",
            "Error": dao.validate().unwrap_err()
        }));
    }

    match create_dao(db, dao).await {
        Ok(result) => HttpResponse::Created().json(json!({
            "message": "Creating DAO",
            "ObjectId": result
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "message": "Failed to create DAO",
            "Error": e.to_string()
        }))
    }
}

/// Get all DAOs
/// 
/// This endpoint returns a list of all DAOs
/// 
/// # API Endpoint
/// ```not_rust
/// GET /dao/all_daos
/// ```
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///    "name": "DAO Name",
///    "logo": "https://www.example.com/logo.png",
///    "id": "507f1f77bcf86cd799439011"
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// Returned when fetching fails:
/// 
/// ```json
/// {
///    "message": "Failed to get all DAOs",
///    "Error": "Database error message"
/// }
/// ```
#[get("/dao/all_daos")]
async fn find_all_daos(db: web::Data<Repository<Dao>>) -> impl Responder {
    let daos = get_all_daos(db).await;
    match daos {
        Ok(result) => {
            HttpResponse::Ok().json(result)
        }
        Err(e) => HttpResponse::BadRequest().json(json!({
          "message": "Failed to get all DAOs",
          "Error": e.to_string()
        })),
    }
}

/// Get DAO by ID
/// 
/// # API Endpoint
/// ```not_rust
/// GET /dao/{id}
/// ```
/// 
/// # Path Parameters
/// 
/// - `id`: String (required) - The ID of the DAO
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///   "name": "DAO Name",
///   "logo": "https://www.example.com/logo.png",
///   "id": "507f1f77bcf86cd799439011"
/// }
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// Returned when the ID is invalid:
/// 
/// ```json
/// {
///   "message": "Invalid id"
/// }
/// 
/// Returned when fetching fails:
/// 
/// ```json
/// {
///  "message": "Failed to get DAO by id",
///  "Error": "Database error message"
/// }
/// ```

#[get("/dao/{id}")]
async fn find_by_id(db: web::Data<Repository<Dao>>, path: Path<String>) -> impl Responder {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }
    let dao = match get_dao_by_id(db, &id).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get DAO by id",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(dao)
}

/// Delete DAO by ID
/// 
/// # API Endpoint
/// ```not_rust
/// DELETE /dao/{id}
/// ```
/// 
/// # Path Parameters
/// 
/// - `id`: String (required) - The ID of the DAO
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///  "message": "Deleted DAO"
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// Returned when the ID is invalid:
/// 
/// ```json
/// {
/// "message": "Invalid id"
/// }
/// ```
/// 
/// Returned when deletion fails:
/// 
/// ```json
/// {
/// "message": "Failed to delete DAO",
/// "Error": "Database error message"
/// }
/// ```

#[delete("/dao/{id}")]
async fn delete_dao(db: web::Data<Repository<Dao>>, path: Path<String>) -> impl Responder {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }
    match delete_by_id(db, &id).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to delete DAO",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(json!({
        "message": "Deleted DAO",
    }))
}

/// Update DAO by ID
/// 
/// # API Endpoint
/// ```not_rust
/// PUT /dao/{id}
/// Content-Type: application/json
/// ```
/// 
/// # Path Parameters
/// 
/// - `id`: String (required) - The ID of the DAO
/// 
/// # Request Body
/// 
/// The request must include a JSON body with the following fields:
/// 
/// ```json
/// {
///   "name": "DAO Name",
///   "description": "DAO Description",
///   "logo": "https://www.example.com/logo.png"
/// }
/// 
/// # Validation Rules
/// 
/// Are the same as the create endpoint
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///  "message": "Updated DAO"
/// }
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// Returned when the ID is invalid:
/// 
/// ```json
/// {
/// "message": "Invalid id"
/// }
/// 
/// And other error responses are the same as the create endpoint

#[put("/dao/{id}")]
async fn update_dao(
    db: web::Data<Repository<Dao>>,
    path: Path<String>,
    dao: web::Json<CreateDaoDto>,
) -> impl Responder {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    if dao.validate().is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Invalid input",
            "Error": dao.validate().unwrap_err()
        }));
    }

    match update_dao_by_id(db, &id, dao.into_inner()).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to update DAO",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(json!({
        "message": "Updated DAO",
    }))
}
