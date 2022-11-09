use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::fs;

pub struct SqliteDatabaseV1_0_0 {
    pub pool: SqlitePool,
}

impl SqliteDatabaseV1_0_0 {
    pub async fn db_connection(source_database_file: &str) -> Self {
        let source_database_connect_url = format!("sqlite://{}?mode=rwc", source_database_file);
        SqliteDatabaseV1_0_0::new(&source_database_connect_url).await
    }

    pub async fn new(database_url: &str) -> Self {
        let db = SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool.");
        Self { pool: db }
    }

    /// Execute migrations for database in version v1.0.0
    pub async fn migrate(&self, fixtures_dir: &str) {
        let migrations_dir = format!("{}database/v1.0.0/migrations/", fixtures_dir);

        let migrations = vec![
            "20210831113004_torrust_users.sql",
            "20210904135524_torrust_tracker_keys.sql",
            "20210905160623_torrust_categories.sql",
            "20210907083424_torrust_torrent_files.sql",
            "20211208143338_torrust_users.sql",
            "20220308083424_torrust_torrents.sql",
            "20220308170028_torrust_categories.sql",
        ];

        for migration_file_name in &migrations {
            let migration_file_path = format!("{}{}", &migrations_dir, &migration_file_name);
            self.run_migration_from_file(&migration_file_path).await;
        }
    }

    async fn run_migration_from_file(&self, migration_file_path: &str) {
        println!("Executing migration: {:?}", migration_file_path);

        let sql = fs::read_to_string(migration_file_path)
            .expect("Should have been able to read the file");

        let res = sqlx::query(&sql).execute(&self.pool).await;

        println!("Migration result {:?}", res);
    }
}
