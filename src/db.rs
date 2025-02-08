use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use tokio::time::{timeout, Duration};
use log::{info, error};

pub async fn establish_connection_pool() -> Result<DatabaseConnection, DbErr> {
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            error!(" DATABASE_URL must be set in the .env file");
            return Err(DbErr::Custom("DATABASE_URL is missing".to_string()));
        }
    };

    info!("🔄 Connecting to database at: {}", database_url);
    info!("⚡ Attempting database connection...");

    let db_result = timeout(Duration::from_secs(10), Database::connect(&database_url)).await;

    match db_result {
        Ok(Ok(db)) => {
            info!(" Database connection successful!");
            Ok(db)
        }
        Ok(Err(e)) => {
            error!(" Database connection failed: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("⏳ Database connection timed out after 10 seconds!");
            Err(DbErr::Custom("Database connection timed out".to_string()))
        }
    }
}
