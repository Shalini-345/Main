use actix_web::{post, web, HttpResponse, Error};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::{info, error};
use serde::{Deserialize, Serialize};

use crate::entities::userentity::{self, ActiveModel, Entity};

#[derive(Debug, Deserialize, Serialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    info!("Received user registration request for username: {}", new_user.username);

    // ✅ CHECK if database connection is valid by running a simple query
    if let Err(err) = Entity::find().one(db.as_ref()).await {
        error!("Database connection error: {:?}", err);
        return Ok(HttpResponse::InternalServerError().body("Database connection is not available"));
    }

    // ✅ HASH password safely
    let password = match hash(new_user.password.clone(), DEFAULT_COST) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Password hashing failed: {:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Failed to hash password"));
        }
    };

    let new_user_active_model = ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password: sea_orm::ActiveValue::Set(password),
        ..Default::default()
    };

    // ✅ CHECK if the user already exists
    match Entity::find()
        .filter(userentity::Column::Username.eq(&new_user.username))
        .one(db.as_ref())
        .await
    {
        Ok(Some(_)) => {
            info!("User already exists: {}", new_user.username);
            return Ok(HttpResponse::Conflict().body("User already exists"));
        }
        Ok(None) => {
            info!("User does not exist, proceeding with registration...");
        }
        Err(err) => {
            error!("Database error occurred while checking user existence: {:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Database error"));
        }
    }

    // ✅ INSERT new user
    match new_user_active_model.insert(db.as_ref()).await {
        Ok(_) => {
            info!("User registered successfully: {}", new_user.username);
            Ok(HttpResponse::Created().body("User registered successfully"))
        }
        Err(err) => {
            error!("Database error while inserting user: {:?}", err);
            Ok(HttpResponse::InternalServerError().body("Error registering user"))
        }
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    info!("Received login request for username: {}", login_data.username);

    // ✅ CHECK if database is available by running a simple query
    if let Err(err) = Entity::find().one(db.as_ref()).await {
        error!("Database connection error: {:?}", err);
        return Ok(HttpResponse::InternalServerError().body("Database connection is not available"));
    }

    match Entity::find()
        .filter(userentity::Column::Username.eq(&login_data.username))
        .one(db.as_ref())
        .await
    {
        Ok(Some(user)) => {
            info!("User found: {}", login_data.username);
            match verify(&login_data.password, &user.password) {
                Ok(true) => {
                    info!("Login successful for user: {}", login_data.username);
                    Ok(HttpResponse::Ok().body("Login successful"))
                }
                Ok(false) => {
                    info!("Invalid credentials for user: {}", login_data.username);
                    Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
                }
                Err(err) => {
                    error!("Error verifying password: {:?}", err);
                    Ok(HttpResponse::InternalServerError().body("Error verifying password"))
                }
            }
        }
        Ok(None) => {
            info!("User not found: {}", login_data.username);
            Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
        }
        Err(err) => {
            error!("Database error while fetching user data: {:?}", err);
            Ok(HttpResponse::InternalServerError().body("Database error"))
        }
    }
}
