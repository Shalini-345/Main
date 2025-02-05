use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use dotenv::dotenv;

pub async fn establish_connection_pool() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    println!("Connecting to database at: {}", database_url); // Debug output

    // Establish connection
    match Database::connect(&database_url).await {
        Ok(db) => {
            println!("✅ Database connection successful!");
            Ok(db)
        }
        Err(e) => {
            eprintln!("❌ Database connection failed: {}", e);
            Err(e)
        }
    }
}
