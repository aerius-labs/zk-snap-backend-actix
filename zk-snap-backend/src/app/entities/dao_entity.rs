use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Dao {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // #[serde(rename = "id")]
    // pub external_id: String,
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "logo")]
    pub logo: Option<String>,

    // #[serde(rename = "membersRoot")]
    // pub members_root: String,
    #[serde(rename = "members")]
    pub members: Vec<String>,
    // #[serde(rename = "membersTree")]
    // pub members_tree: String,
}