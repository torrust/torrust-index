use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AddCategoryForm {
    pub name: String,
    pub icon: Option<String>,
}

pub type DeleteCategoryForm = AddCategoryForm;
