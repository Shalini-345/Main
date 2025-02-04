use actix_web::{post, web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::models::{User, NewUser};
use crate::schema::users;
use crate::db::establish_connection_pool;
use reqwest::Client;
use serde_json::Value;  

#[post("/users/register")]
async fn register_user(new_user: web::Json<NewUser>) -> impl Responder {
    let conn = establish_connection();

  
    let new_user = NewUser {
        username: new_user.username.clone(),
        email: new_user.email.clone(),
        password: new_user.password.clone(),
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user) 
        .get_result::<User>(&conn)
        .expect("Error saving new user");

    HttpResponse::Created().json(user) 
}

// Login the user using Padlock API
#[post("/users/login")]
async fn login_user(login_data: web::Json<NewUser>) -> impl Responder {
    let client = Client::new();
    let padlock_url = "https://api.padlock.com/api/v1/login";
    
    let response = client
        .post(padlock_url)
        .json(&login_data)  
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                
                match res.json::<Value>().await {
                    Ok(json) => {
                        if let Some(token) = json["token"].as_str() {
                            HttpResponse::Ok().json(token) 
                        } else {
                            HttpResponse::Unauthorized().body("Token not found in response")
                        }
                    }
                    Err(_) => HttpResponse::Unauthorized().body("Error parsing Padlock response"),
                }
            } else {
                HttpResponse::Unauthorized().body("Invalid login credentials")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to contact Padlock API"),
    }
}
