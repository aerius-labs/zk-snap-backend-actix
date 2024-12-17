use bson::{doc, Document};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDaoDto {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Name must be between 3 and 50 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 3,
        max = 200,
        message = "Description must be between 3 and 200 characters"
    ))]
    pub description: String,

    #[validate(length(min = 3, message = "Logo URL must be between 3 and 200 characters"))]
    pub logo: Option<String>,

    // #[validate(length(min = 1))]
    // pub members: Vec<String>,
}

/// DTO for response
#[derive(Serialize, Deserialize)]
pub struct DaoResponseDto {
    pub name: String,
    pub logo: String,
    pub id: String,
    // pub members_count: usize,
}

/// DTO for projected fields
#[derive(Serialize, Deserialize, Debug)]
pub struct DaoProjectedFields {
    #[serde(rename = "_id")]
    pub id: Option<bson::oid::ObjectId>,
    pub name: String,
    pub logo: Option<String>,
}

/// Mongo Document for projected fields
impl DaoProjectedFields {
    pub fn projection_doc() -> Vec<Document> {
        vec![
            // Project only the fields we need
            doc! {
                "$project": {
                    "_id": 1,
                    "name": 1,
                    "logo": 1
                }
            },
            // Transform the document to match our desired format
            doc! {
                "$addFields": {
                    "id": { "$toString": "$_id" }
                }
            }
        ]
    }
}