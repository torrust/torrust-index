use std::sync::Arc;

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::{CategoryRecordV2, SqliteDatabaseV2_0_0};

pub async fn transfer_categories(source_database: Arc<SqliteDatabaseV1_0_0>, target_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Transferring categories ...");

    let source_categories = source_database.get_categories_order_by_id().await.unwrap();
    println!("[v1] categories: {:?}", &source_categories);

    let result = target_database.reset_categories_sequence().await.unwrap();
    println!("[v2] reset categories sequence result: {:?}", result);

    for cat in &source_categories {
        println!("[v2] adding category {:?} with id {:?} ...", &cat.name, &cat.category_id);
        let id = target_database
            .insert_category(&CategoryRecordV2 {
                category_id: cat.category_id,
                name: cat.name.clone(),
            })
            .await
            .unwrap();

        assert!(
            id == cat.category_id,
            "Error copying category {:?} from source DB to the target DB",
            &cat.category_id
        );

        println!("[v2] category: {:?} {:?} added.", id, &cat.name);
    }

    let target_categories = target_database.get_categories().await.unwrap();
    println!("[v2] categories: {:?}", &target_categories);
}
