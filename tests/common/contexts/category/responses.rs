use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddedCategoryResponse {
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub data: Vec<ListItem>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ListItem {
    pub category_id: i64,
    pub name: String,
    pub num_torrents: i64,
}
