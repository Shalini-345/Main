use actix_web::{web, App, HttpResponse, HttpServer, post, ResponseError};
use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, QueryFilter, ColumnTrait};
use bcrypt::{hash, verify};
use std::sync::Arc;
use log::{info, error};
use std::fmt;

mod db;
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NewUserForm {
    pub username: String,
    pub password: String,
    pub email: String,
}

// Custom Error Type for HttpResponse
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

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUserForm>,
    pool: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, actix_web::Error> {
    info!("Received user registration request for username: {}", new_user.username);

    let db = pool.get_ref();
    let hashed_password = hash(new_user.password.clone(), 4)
        .map_err(|_| AppError {
            message: "Password hashing failed".to_string(),
        })?;

    let new_user_active_model = entities::userentity::ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password_hash: sea_orm::ActiveValue::Set(hashed_password),
        ..Default::default()
    };

    match entities::userentity::Entity::find()
        .filter(entities::userentity::Column::Username.eq(&new_user.username))
        .one(db.as_ref()).await
    {
        Ok(Some(_)) => Ok(HttpResponse::BadRequest().body("User already exists")),
        Ok(None) => {
            new_user_active_model.insert(db.as_ref()).await
                .map_err(|_| AppError {
                    message: "Error registering user".to_string(),
                })?;
            Ok(HttpResponse::Created().body("User registered successfully"))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Database error")),
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUserForm>,
    pool: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, actix_web::Error> {
    info!("Received login request for username: {}", login_data.username);

    let db = pool.get_ref();

    match entities::userentity::Entity::find()
        .filter(entities::userentity::Column::Username.eq(&login_data.username))
        .one(db.as_ref()).await
    {
        Ok(Some(user)) if verify(&login_data.password, &user.password_hash).unwrap() => {
            Ok(HttpResponse::Ok().body("Login successful"))
        }
        _ => Ok(HttpResponse::Unauthorized().body("Invalid credentials")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Initialize logging

    info!("Starting application...");

    let pool = match db::establish_connection_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to establish database connection: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed"));
        }
    };

    info!("Database connection established successfully!");

    info!("Starting Actix server on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register_user)
            .service(login_user)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    info!("Server is running at http://0.0.0.0:8080");

    Ok(())
}
