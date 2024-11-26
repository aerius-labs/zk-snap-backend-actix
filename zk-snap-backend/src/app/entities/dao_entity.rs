use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Dao {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "logo")]
    pub logo: Option<String>,
}
