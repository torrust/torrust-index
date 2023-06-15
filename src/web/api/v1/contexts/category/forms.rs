use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CategoryForm {
    pub name: String,
    pub icon: Option<String>,
}
