use actix_web::{get, post,put , delete, web, HttpResponse, Error, HttpRequest, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait, Condition, Set};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::info;
use serde::{Deserialize, Serialize};
use validator::Validate;
use regex::Regex;
use crate::auth::AuthTokenClaims;
use crate::entities::userentity::{self, ActiveModel, Entity};
use crate::entities::driverentity;
use crate::db::establish_connection_pool;
use serde_json::json;
use chrono::Utc;
use crate::entities::payment::{ActiveModel as PaymentActiveModel, Entity as PaymentEntity};


// user log in api


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

    if let Err(_) = new_user.validate() {
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

    let password_hash = hash(&new_user.password, DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to hash password"))?;

    let new_user_active_model = ActiveModel {
        username: Set(new_user.username.clone()),
        email: Set(new_user.email.clone()),
        password: Set(password_hash),
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


// driver api



#[post("/drivers")]
async fn create_driver(driver: web::Json<driverentity::Model>) -> impl Responder {
    match establish_connection_pool().await {
        Ok(db) => {
            let existing_driver = driverentity::Entity::find()
                .filter(
                    driverentity::Column::Email.eq(driver.email.clone())
                    .or(driverentity::Column::Phone.eq(driver.phone.clone()))
                )
                .one(&db)
                .await;

            match existing_driver {
                Ok(Some(existing)) => {
                    let message = if existing.email == driver.email {
                        "Driver with this email already registered"
                    } else {
                        "Phone number already exists"
                    };
                    
                    let response = json!({
                        "message": message,
                        "email": driver.email,
                        "phone": driver.phone
                    });
                    HttpResponse::Conflict().json(response)
                }
                Ok(None) => {
                    use chrono::Utc;

                    let now = Utc::now().naive_utc(); 

                    let new_driver = driverentity::ActiveModel {
                        first_name: Set(driver.first_name.clone()),
                        last_name: Set(driver.last_name.clone()),
                        email: Set(driver.email.clone()),
                        phone: Set(driver.phone.clone()),
                        photo: Set(driver.photo.clone()),
                        rating: Set(driver.rating),
                        total_rides: Set(driver.total_rides),
                        about_me: Set(driver.about_me.clone()),
                        from_location: Set(driver.from_location.clone()),
                        languages: Set(driver.languages.clone()),
                        is_pilot: Set(driver.is_pilot),
                        license_number: Set(driver.license_number.clone()),
                        verification_status: Set(driver.verification_status.clone()),
                        current_lat: Set(driver.current_lat),
                        current_lng: Set(driver.current_lng),
                        availability_status: Set(driver.availability_status.clone()),
                        created_at: Set(Some(now)), 
                        updated_at: Set(Some(now)), 
                        ..Default::default() 
                    };

                    match driverentity::Entity::insert(new_driver).exec(&db).await {
                        Ok(inserted) => {
                            let response = json!({
                                "message": "Driver registered successfully!",
                                "driver_id": inserted.last_insert_id, 
                                "email": driver.email,
                                "phone": driver.phone,
                                "created_at": now,
                                "updated_at": now
                            });
                            HttpResponse::Created().json(response)
                        }
                        Err(e) => {
                            eprintln!("Database insertion error: {:?}", e);
                            HttpResponse::InternalServerError().body("Error creating driver")
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Database query failed: {:?}", e);
                    HttpResponse::InternalServerError().body("Database query failed")
                }
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Database connection failed"),
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[get("/drivers")]
async fn get_drivers() -> impl Responder {
    println!("Received request at /drivers"); 

    match establish_connection_pool().await {
        Ok(db) => {
            match driverentity::Entity::find().all(&db).await {
                Ok(drivers) => {
                    if drivers.is_empty() {
                        println!("No drivers found in the database.");
                        HttpResponse::Ok().json(json!({"message": "No drivers found", "drivers": []}))
                    } else {
                        println!("Fetched {} drivers", drivers.len());
                        HttpResponse::Ok().json(drivers)
                    }
                }
                Err(e) => {
                    eprintln!(" Error fetching drivers: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({"error": "Failed to fetch drivers"}))
                }
            }
        }
        Err(e) => {
            eprintln!(" Database connection failed: {:?}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Database connection failed"}))
        }
    }
}




pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_drivers);
    cfg.service(create_driver);

}

// payment API





#[derive(Serialize, Deserialize)]
pub struct PaymentRequest {
    pub user_id: i32,
    pub payment_type: String,
    pub card_number: Option<String>,
    pub card_holder: Option<String>,
    pub expiry_month: Option<i16>,
    pub expiry_year: Option<i16>,
    pub card_type: Option<String>,
    pub is_default: bool,
    pub paypal_email: Option<String>,
}

/// GET API - Fetch all payments
#[get("/payments")]
async fn get_payments(db: web::Data<DatabaseConnection>) -> impl Responder {
    match PaymentEntity::find().all(db.get_ref()).await {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(e) => {
            eprintln!("Error fetching payments: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch payments: {:?}", e)
            }))
        }
    }
}

/// POST API - Create a new payment
#[post("/payments")]
async fn create_payment(
    db: web::Data<DatabaseConnection>,
    payment_data: web::Json<PaymentRequest>,
) -> impl Responder {
    let new_payment = PaymentActiveModel {
        user_id: Set(payment_data.user_id),
        payment_type: Set(payment_data.payment_type.clone()),
        card_number: Set(payment_data.card_number.clone()),
        card_holder: Set(payment_data.card_holder.clone()),
        expiry_month: Set(payment_data.expiry_month),
        expiry_year: Set(payment_data.expiry_year),
        card_type: Set(payment_data.card_type.clone()),
        is_default: Set(payment_data.is_default),
        paypal_email: Set(payment_data.paypal_email.clone()),
        created_at: Set(Some(Utc::now())),
        updated_at: Set(Some(Utc::now())),
        ..Default::default()
    };

    match new_payment.insert(db.get_ref()).await {
        Ok(inserted) => HttpResponse::Created().json(serde_json::json!({
            "message": "Payment created successfully",
            "payment": inserted
        })),
        Err(e) => {
            eprintln!("Error inserting payment: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create payment: {:?}", e)
            }))
        }
    }
}

/// PUT API - Update a payment
#[put("/payments/{id}")]
async fn update_payment(
    db: web::Data<DatabaseConnection>,
    payment_id: web::Path<i32>,
    payment_data: web::Json<PaymentRequest>,
) -> impl Responder {
    let id = payment_id.into_inner();
    
    match PaymentEntity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(existing_payment)) => {
            let mut updated_payment: PaymentActiveModel = existing_payment.into();
            updated_payment.payment_type = Set(payment_data.payment_type.clone());
            updated_payment.card_number = Set(payment_data.card_number.clone());
            updated_payment.card_holder = Set(payment_data.card_holder.clone());
            updated_payment.expiry_month = Set(payment_data.expiry_month);
            updated_payment.expiry_year = Set(payment_data.expiry_year);
            updated_payment.card_type = Set(payment_data.card_type.clone());
            updated_payment.is_default = Set(payment_data.is_default);
            updated_payment.paypal_email = Set(payment_data.paypal_email.clone());
            updated_payment.updated_at = Set(Some(Utc::now()));

            match updated_payment.update(db.get_ref()).await {
                Ok(updated) => HttpResponse::Ok().json(serde_json::json!({
                    "message": "Payment updated successfully",
                    "payment": updated
                })),
                Err(e) => {
                    eprintln!("Error updating payment: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to update payment: {:?}", e)
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment not found. Please check the payment ID."
        })),
        Err(e) => {
            eprintln!("Database error while updating payment: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {:?}", e)
            }))
        }
    }
}

/// DELETE API - Delete a payment
#[delete("/payments/{id}")]
async fn delete_payment(
    db: web::Data<DatabaseConnection>,
    payment_id: web::Path<i32>,
) -> impl Responder {
    let id = payment_id.into_inner();

    match PaymentEntity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(existing_payment)) => {
            let active_payment: PaymentActiveModel = existing_payment.into();

            match active_payment.delete(db.get_ref()).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Payment with ID {} deleted successfully", id)
                })),
                Err(e) => {
                    eprintln!("Error deleting payment: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to delete payment: {:?}", e)
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment not found. Please check the payment ID."
        })),
        Err(e) => {
            eprintln!("Database error while deleting payment: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {:?}", e)
            }))
        }
    }
}

/// Initialize payment routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_payments)
       .service(create_payment)
       .service(update_payment)
       .service(delete_payment);
}

