use serde::{Deserialize, Serialize};

use crate::databases::database::Category as DatabaseCategory;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i64,
    // Deprecated. Use `id`.
    pub category_id: i64,
    pub name: String,
    pub num_torrents: i64,
}

#[allow(clippy::module_name_repetitions)]
pub type CategoryId = i64;

impl From<DatabaseCategory> for Category {
    fn from(db_category: DatabaseCategory) -> Self {
        Category {
            id: db_category.category_id,
            category_id: db_category.category_id,
            name: db_category.name,
            num_torrents: db_category.num_torrents,
        }
    }
}
