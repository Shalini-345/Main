use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait};
use crate::entities::userentity::{self, ActiveModel, Entity};
use crate::models::NewUser;
use bcrypt::{hash, verify, DEFAULT_COST};
use log::error;

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>
) -> impl Responder {
    let password_hash = match hash(new_user.password.clone(), DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let new_user_active_model = ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password_hash: sea_orm::ActiveValue::Set(password_hash),
        ..Default::default()
    };

    match userentity::Entity::find()
        .filter(userentity::Column::Username.eq(&new_user.username))
        .one(db.as_ref()).await
    {
        Ok(Some(_)) => HttpResponse::Conflict().body("User already exists"), // Changed to 409 Conflict
        Ok(None) => {
            match new_user_active_model.insert(db.as_ref()).await {
                Ok(_) => HttpResponse::Created().body("User registered successfully"),
                Err(_) => {
                    error!("Error inserting user into the database.");
                    HttpResponse::InternalServerError().body("Error registering user")
                }
            }
        }
        Err(_) => {
            error!("Database error occurred while checking user existence.");
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    match userentity::Entity::find()
        .filter(userentity::Column::Username.eq(&login_data.username))
        .one(db.as_ref()).await
    {
        Ok(Some(user)) => {
            match verify(&login_data.password, &user.password_hash) {
                Ok(true) => HttpResponse::Ok().body("Login successful"),
                Ok(false) => HttpResponse::Unauthorized().body("Invalid credentials"),
                Err(_) => HttpResponse::InternalServerError().body("Error verifying password"),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => {
            error!("Error querying the database.");
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}
