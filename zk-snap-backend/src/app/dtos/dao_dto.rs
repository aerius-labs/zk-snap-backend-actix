use std::vec;

use bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::app::repository::traits::Projectable;

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

/// Projectable trait implementation for DAO
impl Projectable for DaoResponseDto {
    fn get_projection_pipeline(id: Option<ObjectId>) -> Vec<Document> {
        let obj_id = match id {
            Some(id) => id,
            None => ObjectId::new(),
        };

        vec![
            // Match the document by id
            doc! {
                "$match": {
                    "_id": obj_id
                }
            },
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