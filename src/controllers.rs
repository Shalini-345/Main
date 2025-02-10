use actix_web::{get, post, web, HttpResponse, Error, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait, Condition};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::info;
use serde::{Deserialize, Serialize};
use validator::Validate;
use regex::Regex;
use crate::auth::AuthTokenClaims;
use crate::entities::userentity::{self, ActiveModel, Entity};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 3))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[\w.-]+@[a-zA-Z\d.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    info!("Received user registration request for username: {}", new_user.username);

    if let Err(_err) = new_user.validate() {
        //let error_messages: Vec<String> = err.field_errors()
           // .iter()
            //.map(|(field, errors)| format!("{}: {}", field, errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")))
            //.collect();

            return Ok(HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid email format" })));
    
    }

    let existing_user = Entity::find()
        .filter(
            Condition::any()
                .add(userentity::Column::Username.eq(&new_user.username))
                .add(userentity::Column::Email.eq(&new_user.email)),
        )
        .one(db.as_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database error while checking user existence"))?;

    if let Some(user) = existing_user {
        let conflict_field = if user.username == new_user.username {
            "username"
        } else {
            "email"
        };
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "error": format!("{} already exists", conflict_field)
        })));
    }

    // Hash password
    let password_hash = hash(&new_user.password, DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to hash password"))?;

    // Insert new user
    let new_user_active_model = ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password: sea_orm::ActiveValue::Set(password_hash),
        ..Default::default()
    };

    match new_user_active_model.insert(db.as_ref()).await {
        Ok(_) => Ok(HttpResponse::Created().json(serde_json::json!({ "message": "User registered successfully" }))),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error registering user" }))),
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    if !is_valid_email(&login_data.email) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid email format" })));
    }

    let user = Entity::find()
        .filter(
            Condition::all()
                .add(userentity::Column::Email.eq(&login_data.email))
                .add(userentity::Column::Username.eq(&login_data.username)),
        )
        .one(db.as_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database error while fetching user"))?;

    let user = match user {
        Some(user) => user,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid credentials" }))),
    };

    let is_password_valid = verify(&login_data.password, &user.password)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Error verifying password"))?;

    if !is_password_valid {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid credentials" })));
    }

    let access_token = AuthTokenClaims::new(user.id, 24).generate_token()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Access token generation failed"))?;

    let refresh_token = AuthTokenClaims::new(user.id, 168).generate_token()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Refresh token generation failed"))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token
    })))
}

#[get("/users")]
async fn get_users(db: web::Data<DatabaseConnection>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if let Ok(_) = AuthTokenClaims::validate_token(token) {
                    let users = Entity::find()
                        .all(db.as_ref())
                        .await
                        .map_err(|_| actix_web::error::ErrorInternalServerError("Error fetching users"))?
                        .into_iter()
                        .map(|user| serde_json::json!({
                            "id": user.id,
                            "username": user.username,
                            "email": user.email,
                        }))
                        .collect::<Vec<_>>();

                    return Ok(HttpResponse::Ok().json(users));
                }
            }
        }
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid token" })));
    }

    Ok(HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Missing token" })))
}
