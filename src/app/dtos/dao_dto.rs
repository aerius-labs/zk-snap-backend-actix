use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDaoDto {
    #[validate(length(min = 3, max = 50))]
    pub name: String,

    #[validate(length(min = 3, max = 200))]
    pub description: String,

    #[validate(length(min = 3))]
    pub logo: Option<String>,

    #[validate(length(min = 1))]
    pub members: Vec<String>,
}
