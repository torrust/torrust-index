use serde::Serialize;

#[derive(Serialize)]
pub struct AddTagForm {
    pub name: String,
}

#[derive(Serialize)]
pub struct DeleteTagForm {
    pub tag_id: i64,
}
