use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use std::sync::Arc;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::CategoryRecordV1;

pub struct CategoryTester {
    source_database: Arc<SqliteDatabaseV1_0_0>,
    destiny_database: Arc<SqliteDatabaseV2_0_0>,
    test_data: TestData,
}

pub struct TestData {
    pub categories: Vec<CategoryRecordV1>,
}

impl CategoryTester {
    pub fn new(
        source_database: Arc<SqliteDatabaseV1_0_0>,
        destiny_database: Arc<SqliteDatabaseV2_0_0>,
    ) -> Self {
        let category_01 = CategoryRecordV1 {
            category_id: 10,
            name: "category name 10".to_string(),
        };
        let category_02 = CategoryRecordV1 {
            category_id: 11,
            name: "category name 11".to_string(),
        };

        Self {
            source_database,
            destiny_database,
            test_data: TestData {
                categories: vec![category_01, category_02],
            },
        }
    }

    pub fn get_valid_category_id(&self) -> i64 {
        self.test_data.categories[0].category_id
    }

    /// Table `torrust_categories`
    pub async fn load_data_into_source_db(&self) {
        // Delete categories added by migrations
        self.source_database.delete_all_categories().await.unwrap();

        // Add test categories
        for categories in &self.test_data.categories {
            self.source_database
                .insert_category(&categories)
                .await
                .unwrap();
        }
    }

    /// Table `torrust_categories`
    pub async fn assert_data_in_destiny_db(&self) {
        for categories in &self.test_data.categories {
            let imported_category = self
                .destiny_database
                .get_category(categories.category_id)
                .await
                .unwrap();

            assert_eq!(imported_category.category_id, categories.category_id);
            assert_eq!(imported_category.name, categories.name);
        }
    }
}
