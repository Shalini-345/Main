use sea_orm::{DatabaseConnection, DbErr};
use std::env;
use dotenv::dotenv;

pub async fn establish_connection_pool() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();

    // Load DATABASE_URL from .env file
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    println!("Connecting to database: {}", database_url); // Debugging output

    // Establish the database connection using sea_orm
    sea_orm::Database::connect(&database_url).await
}
