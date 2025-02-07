use actix_web::{post, web, HttpResponse, Error};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::{info, error};
use serde::{Deserialize, Serialize}; // Ensure Deserialize is imported

use crate::entities::userentity::{self, ActiveModel, Entity};

#[derive(Debug, Deserialize, Serialize)] // Ensure NewUser is properly defined
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {  // Explicit return type
    info!("Received user registration request for username: {}", new_user.username);

    let password_hash = match hash(new_user.password.clone(), DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            error!("Password hashing failed");
            return Ok(HttpResponse::InternalServerError().body("Failed to hash password"));
        }
    };

    let new_user_active_model = ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password_hash: sea_orm::ActiveValue::Set(password_hash),
        ..Default::default()
    };

    info!("Checking if user already exists...");
    match Entity::find()
        .filter(userentity::Column::Username.eq(&new_user.username))
        .one(db.as_ref()).await
    {
        Ok(Some(_)) => {
            info!("User already exists");
            Ok(HttpResponse::Conflict().body("User already exists"))
        }
        Ok(None) => {
            info!("Inserting new user into the database...");
            match new_user_active_model.insert(db.as_ref()).await {
                Ok(_) => {
                    info!("User registered successfully");
                    Ok(HttpResponse::Created().body("User registered successfully"))
                }
                Err(_) => {
                    error!("Database error while inserting user");
                    Ok(HttpResponse::InternalServerError().body("Error registering user"))
                }
            }
        }
        Err(_) => {
            error!("Database error occurred while checking user existence");
            Ok(HttpResponse::InternalServerError().body("Database error"))
        }
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {  // Explicit return type
    info!("Received login request for username: {}", login_data.username);

    match Entity::find()
        .filter(userentity::Column::Username.eq(&login_data.username))
        .one(db.as_ref()).await
    {
        Ok(Some(user)) => {
            match verify(&login_data.password, &user.password_hash) {
                Ok(true) => {
                    info!("Login successful for user: {}", login_data.username);
                    Ok(HttpResponse::Ok().body("Login successful"))
                }
                Ok(false) => {
                    info!("Invalid credentials for user: {}", login_data.username);
                    Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
                }
                Err(_) => {
                    error!("Error verifying password");
                    Ok(HttpResponse::InternalServerError().body("Error verifying password"))
                }
            }
        }
        Ok(None) => {
            info!("User not found: {}", login_data.username);
            Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
        }
        Err(_) => {
            error!("Database error while fetching user data");
            Ok(HttpResponse::InternalServerError().body("Database error"))
        }
    }
}
