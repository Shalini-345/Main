use sea_orm::{Database, DatabaseConnection, DbErr};
use dotenv::dotenv;
use std::env;
use tokio::time::{timeout, Duration};

pub async fn establish_connection_pool() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok(); // ✅ Load .env variables

    let database_url = env::var("DATABASE_URL").expect("❌ DATABASE_URL must be set in the .env file");

    println!("🔄 Connecting to database at: {}", database_url);
    println!("⚡ Attempting database connection...");

    let db_result = timeout(Duration::from_secs(10), Database::connect(&database_url)).await;

    match db_result {
        Ok(Ok(db)) => {
            println!("✅ Database connection successful!");
            Ok(db)
        }
        Ok(Err(e)) => {
            eprintln!("❌ Database connection failed: {}", e);
            Err(e)
        }
        Err(_) => {
            eprintln!("⏳ Database connection timed out after 10 seconds!");
            Err(DbErr::Custom("Database connection timed out".to_string()))
        }
    }
}
