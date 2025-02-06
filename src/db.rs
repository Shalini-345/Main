use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use dotenv::dotenv;
use tokio::time::{timeout, Duration};

/// Establishes a connection pool to the database and includes timeout handling.
pub async fn establish_connection_pool() -> Result<DatabaseConnection, DbErr> {
    // Load environment variables from the .env file
    dotenv().ok();

    // Retrieve the database URL from environment variables or panic if not found
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("❌ DATABASE_URL must be set in the .env file");
        std::process::exit(1); // Exit if the DATABASE_URL is missing
    });

    // Print message indicating the start of the connection process
    println!("🔄 Connecting to database at: {}", database_url);
    println!("⚡ Attempting database connection...");

    // Attempt to connect to the database with a 5-second timeout
    let db_result = timeout(Duration::from_secs(5), Database::connect(&database_url)).await;

    // Handle the result of the connection attempt
    match db_result {
        // Successfully connected to the database
        Ok(Ok(db)) => {
            println!("✅ Database connection successful!");
            Ok(db)
        }
        // Failed to connect to the database
        Ok(Err(e)) => {
            eprintln!("❌ Database connection failed: {}", e);
            Err(e)
        }
        // Connection attempt timed out after 5 seconds
        Err(_) => {
            eprintln!("⏳ Database connection timed out after 5 seconds!");
            Err(DbErr::Custom("Database connection timed out".to_string()))
        }
    }
}
