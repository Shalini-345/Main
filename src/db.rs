use sea_orm::{Database, DatabaseConnection, DbErr, RuntimeErr};
use std::env;
use dotenv::dotenv;
use tokio::time::{timeout, Duration};

pub async fn establish_connection_pool() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok(); 

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("❌ DATABASE_URL must be set in the .env file");
        std::process::exit(1);
    });

    println!("🔄 Connecting to database at: {}", database_url); 
    println!("⚡ Attempting database connection...");

    let db_result = timeout(Duration::from_secs(5), Database::connect(&database_url)).await;

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
            eprintln!("⏳ Database connection timed out after 5 seconds!");
            Err(DbErr::Conn(RuntimeErr::Internal("Database connection timed out".to_string())))
        }
    }
}
