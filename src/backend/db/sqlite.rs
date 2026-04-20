use sea_orm::{Database, DatabaseConnection};

type Result<T> = color_eyre::Result<T>;

// TODO: Update path
const DB_PATH: &str = "/home/***REMOVED***/Projects/randale_iced/test.sqlite";

pub struct DB;

impl DB {
    pub async fn open() -> Result<DatabaseConnection> {
        let db: DatabaseConnection =
            Database::connect(format!("sqlite://{DB_PATH}?mode=rwc")).await?;

        // TODO: Naive check if database works
        db.ping().await?;

        // Synchronizes database schema with entity definitions
        db.get_schema_registry(module_path!().split("::").next().unwrap())
            .sync(&db)
            .await?;

        Ok(db)
    }
}
