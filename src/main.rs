use actix_web::{web, App, HttpResponse, HttpServer, Responder, ResponseError};
use sea_orm::DatabaseConnection;
use log::{info, error};
use std::fmt;
use dotenv::dotenv;
use sea_orm_migration::prelude::*;
use migration::{Migrator, MigratorTrait};
use db::establish_connection_pool;

mod db;
mod controllers;
mod auth;
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
    pub mod cities;
    pub mod userprofile;
}

use controllers::get_users; 
use controllers::{get_all_vehicles, get_vehicle, create_vehicle, delete_vehicle}; 

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

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Arrively API!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); 
    env_logger::init();  

    info!(" Starting the application...");

    let pool = match establish_connection_pool().await {
        Ok(pool) => web::Data::new(pool), 
        Err(e) => {
            error!(" Failed to establish database connection: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed"));
        }
    };

    info!("Database connection established successfully!");

    info!(" Running database migrations...");
    if let Err(err) = run_migrations(pool.get_ref()).await { 
        error!(" Migration failed: {}", err);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Migration failed"));
    }
    info!("Migrations completed successfully!");

    info!("Starting Actix server on 0.0.0.0:8081...");
    info!(" Server is running at http://0.0.0.0:8081");

    HttpServer::new(move || {
        App::new()
        .wrap(actix_web::middleware::Logger::default())  
        .app_data(pool.clone()) 
            .route("/", web::get().to(index)) 

            .service(web::scope("/v1")
            .route("/", web::get().to(index)) 

            .service(controllers::register_user)
            .service(get_users)
            .configure(controllers::configure)
           // .configure(controllers::init)
            .service(get_all_vehicles)
            .service(get_vehicle)
            .service(create_vehicle)
            .service(delete_vehicle) 
            .configure(controllers::config) 
            .service(controllers::get_cities)
            .service(controllers::add_cities) 
            .service(controllers::create_settings)
            .service(controllers::get_settings)
            .service(controllers::update_settings)
            .service(controllers::delete_settings)
            .service(controllers::get_tickets)
            .service(controllers::create_ticket)
            .service(controllers::update_ticket)  
            .service(controllers::delete_ticket)
            .service(controllers::create_user_profile) 
            .service(controllers::get_user_profiles)   
            .service(controllers::update_user_profile) 
            .service(controllers::delete_user_profile)
            .service(controllers::get_recent_locations)
            .service(controllers::add_recent_location)) 
 
    })
    .bind("0.0.0.0:8081")?  
    .run()
    .await?;
    
    Ok(())
}

async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await.map_err(|e| {
        error!(" Migration error: {}", e);
        e
    })
}
