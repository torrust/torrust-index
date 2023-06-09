use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddedTagResponse {
    pub data: String,
}

#[derive(Deserialize)]
pub struct DeletedTagResponse {
    pub data: i64, // tag_id
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub data: Vec<ListItem>,
}

impl ListResponse {
    pub fn find_tag_id(&self, tag_name: &str) -> i64 {
        self.data.iter().find(|tag| tag.name == tag_name).unwrap().tag_id
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ListItem {
    pub tag_id: i64,
    pub name: String,
}
