use actix_web::{web, App, HttpResponse, HttpServer, ResponseError};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use log::{info, error};
use std::fmt;
use dotenv::dotenv;
use sea_orm_migration::prelude::*;
use migration::{Migrator, MigratorTrait};
use db::establish_connection_pool;

mod db;
mod controllers;
mod entities {
    pub mod userentity;
    pub mod faviorate;
    pub mod helpsupport;
    pub mod payment;
    pub mod recentlocation;
    pub mod rideentity;
    pub mod settings;
    pub mod vehicleentity;
    pub mod driverentity;
}

#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body(self.message.clone())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); 
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) 
        .init();

    info!("🚀 Starting the application...");

    let pool = match establish_connection_pool().await {
        Ok(pool) => Arc::new(pool),
        Err(e) => {
            error!(" Failed to establish database connection: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed"));
        }
    };

    info!("✅ Database connection established successfully!");

    // Run pending migrations
    info!("⚡ Running database migrations...");
    if let Err(err) = run_migrations(&*pool).await {
        error!(" Migration failed: {}", err);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Migration failed"));
    }
    info!(" Migrations completed successfully!");

    info!(" Starting Actix server on 0.0.0.0:8081...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(controllers::register_user)
            .service(controllers::login_user)
    })
    .bind("0.0.0.0:8081")?  
    .run()
    .await?;

    info!("Server is running at http://0.0.0.0:8081");

    Ok(())
}

async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await.map_err(|e| {
        error!("Migration error: {}", e);
        e
    })
}
