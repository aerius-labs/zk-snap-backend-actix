use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDaoDto {
    #[validate(length(min = 3, max = 50, message = "Name must be between 3 and 50 characters"))]
    pub name: String,

    #[validate(length(min = 3, max = 200, message = "Description must be between 3 and 200 characters"))]
    pub description: String,

    #[validate(length(min = 3, message = "Logo URL must be between 3 and 200 characters"))]
    pub logo: Option<String>,

    #[validate(length(min = 1))]
    pub members: Vec<String>,
}
