use std::sync::Arc;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, post};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::entities::userentity::{Entity as UserEntity, ActiveModel as UserActiveModel};
use crate::entities::rideentity::Relation;  
use crate::db as my_db;  
use sea_orm::ColumnTrait;

mod entities {
    pub mod userentity;
    pub mod rideentity;
    pub mod driverentity;
    pub mod faviorate;
    pub mod helpsupport;
    pub mod payment;
    pub mod recentlocation;
    pub mod settings;
    pub mod vehicleentity;
}

#[derive(serde::Deserialize, serde::Serialize)] 
pub struct NewUserForm {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUserForm>,
    pool: web::Data<Arc<DatabaseConnection>>,  
) -> impl Responder {
    let db = pool.get_ref();  
    
    let new_user_active_model = UserActiveModel {
        username: sea_orm::ActiveValue::Set(new_user.username.clone()),
        password_hash: sea_orm::ActiveValue::Set(new_user.password.clone()),
        email: sea_orm::ActiveValue::Set(new_user.email.clone()),
        ..Default::default()
    };
    
    match UserEntity::find()
        .filter(crate::entities::userentity::Column::Username.eq(&new_user.username))
        .one(db.as_ref()).await
    {
        Ok(Some(_)) => HttpResponse::Ok().body("User already exists"),
        Ok(None) => {
            UserEntity::insert(new_user_active_model).exec(db.as_ref()).await.unwrap();
            HttpResponse::Ok().body("User registered successfully")
        }
        Err(e) => {
            eprintln!("Error registering user: {}", e);
            HttpResponse::InternalServerError().body("Failed to register user")
        }
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUserForm>,
    pool: web::Data<Arc<DatabaseConnection>>,  
) -> impl Responder {
    let db = pool.get_ref(); 

    match UserEntity::find()
        .filter(crate::entities::userentity::Column::Username.eq(&login_data.username))
        .filter(crate::entities::userentity::Column::PasswordHash.eq(&login_data.password))
        .one(db.as_ref()).await
    {
        Ok(Some(_)) => HttpResponse::Ok().body("Login successful"),
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => HttpResponse::InternalServerError().body("Error during login"),
    }
}

async fn run_orm_sea() -> Result<(), sea_orm::DbErr> {
    println!("Running ORM Sea...");
    Ok(())
}

pub mod db {
    use sea_orm::{Database, DbErr, DatabaseConnection};
    use std::sync::Arc;

    pub async fn establish_connection_pool() -> DatabaseConnection {
        let db_url = "postgres://postgres:shalu11@13.60.21.29:5432/my_database"; 
        Database::connect(db_url).await.unwrap()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running ORM Sea...");
    if let Err(e) = run_orm_sea().await {
        eprintln!("Error running ORM-Sea: {}", e);
    }

    let pool = my_db::establish_connection_pool().await;

    let relation = Relation::User;
    let value = relation as i32;
    println!("The value of User is: {}", value);

    match relation {
        Relation::User => println!("User variant"),
        Relation::Payment => println!("Payment variant"),
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register_user)
            .service(login_user)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
