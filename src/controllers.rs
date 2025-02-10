use actix_web::{get, post, web, HttpResponse, Error, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait, Condition};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::info;
use serde::{Deserialize, Serialize};
use validator::Validate;
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

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    info!("Received user registration request for username: {}", new_user.username);

    if let Err(err) = new_user.validate() {
        return Ok(HttpResponse::BadRequest().json(err));
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

    if existing_user.is_some() {
        return Ok(HttpResponse::Conflict().body("User already exists"));
    }

    let password_hash = hash(&new_user.password, DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to hash password"))?;

    let new_user_active_model = ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password: sea_orm::ActiveValue::Set(password_hash),
        ..Default::default()
    };

    new_user_active_model.insert(db.as_ref()).await
        .map(|_| HttpResponse::Created().body("User registered successfully"))
        .map_err(|_| actix_web::error::ErrorInternalServerError("Error registering user"))
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let user = Entity::find()
        .filter(userentity::Column::Username.eq(&login_data.username))
        .one(db.as_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database error while fetching user"))?;

    let user = match user {
        Some(user) => user,
        None => return Ok(HttpResponse::Unauthorized().body("Invalid credentials")),
    };

    let is_password_valid = verify(&login_data.password, &user.password)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Error verifying password"))?;

    if !is_password_valid {
        return Ok(HttpResponse::Unauthorized().body("Invalid credentials"));
    }

    let claims = AuthTokenClaims::new(user.id);
    let token = claims.generate_token()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Token generation failed"))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })))
}

#[get("/users")]
async fn get_users(db: web::Data<DatabaseConnection>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_header) = auth_header {
        if let Ok(token) = auth_header.to_str() {
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
        return Ok(HttpResponse::Unauthorized().body("Invalid token"));
    }

    Ok(HttpResponse::Unauthorized().body("Missing token"))
}
