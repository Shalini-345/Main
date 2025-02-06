use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait};
use crate::entities::userentity::{self, ActiveModel, Entity};
use crate::models::NewUser;

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>
) -> impl Responder {
    let new_user_active_model = ActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        password_hash: sea_orm::ActiveValue::Set(new_user.password.clone()),
        ..Default::default()
    };

    match userentity::Entity::find()
        .filter(userentity::Column::Username.eq(&new_user.username))
        .one(db.as_ref()).await
    {
        Ok(Some(_)) => HttpResponse::Ok().body("User already exists"),
        Ok(None) => {
            new_user_active_model.insert(db.as_ref()).await.unwrap();
            HttpResponse::Created().body("User registered successfully")
        }
        Err(_) => HttpResponse::InternalServerError().body("Error"),
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
            if bcrypt::verify(&login_data.password, &user.password_hash).unwrap() {
                HttpResponse::Ok().body("Login successful")
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => HttpResponse::InternalServerError().body("Error"),
    }
}
