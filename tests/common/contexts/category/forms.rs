use serde::Serialize;

#[derive(Serialize)]
pub struct AddCategoryForm {
    pub name: String,
    pub icon: Option<String>,
}

pub type DeleteCategoryForm = AddCategoryForm;
