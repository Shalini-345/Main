use actix_web::{delete, get, post,put, web, HttpRequest, HttpResponse, Responder};
use regex::Regex;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set };
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use crate::auth::AuthTokenClaims;
use crate::entities::userentity::{self, Entity as UserEntity};
use crate::entities::{self, driverentity, vehicleentity};
use crate::db::establish_connection_pool;
use serde_json::json;
//use chrono::Utc;
//use crate::entities::payment::{ActiveModel as PaymentActiveModel, Entity as PaymentEntity};
use crate::entities::rideentity::{self, Entity as RideEntity};
use rust_decimal::Decimal;
use chrono::{DateTime as ChronoDateTime, Utc};
use crate::entities::settings::{self};
use log::{error, info};
use crate::entities::helpsupport::NewTicketRequest;

use actix_web::Error;
use crate::entities::cities::{self};


//user registration


#[derive(Deserialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub city: i32,
    pub phone_number: String,
}

fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[\w.-]+@[a-zA-Z\d.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

fn validate_phone(phone: &str) -> Result<(), String> {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    if phone_regex.is_match(phone) {
        Ok(())
    } else {
        Err("Invalid phone number format".to_string())
    }
}

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    if !is_valid_email(&new_user.email) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Incorrect email format"
        })));
    }

    if let Err(_) = validate_phone(&new_user.phone_number) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid phone number format"
        })));
    }

    let existing_email = UserEntity::find()
        .filter(userentity::Column::Email.eq(new_user.email.clone()))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            eprintln!("Database query error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if existing_email.is_some() {
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "error": "Email already exists"
        })));
    }

    let existing_phone = UserEntity::find()
        .filter(userentity::Column::PhoneNumber.eq(new_user.phone_number.clone()))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            eprintln!("Database query error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if existing_phone.is_some() {
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "error": "Phone number already exists"
        })));
    }

    let password_hash = hash(&new_user.password, DEFAULT_COST).map_err(|e| {
        eprintln!("Password hashing error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to hash password")
    })?;

    let new_user_active_model = userentity::ActiveModel {
        first_name: Set(new_user.first_name.clone()),
        last_name: Set(new_user.last_name.clone()),
        email: Set(new_user.email.clone()),
        password: Set(password_hash),
        city: Set(new_user.city),
        phone_number: Set(new_user.phone_number.clone()),
        ..Default::default()
    };

    match new_user_active_model.insert(db.as_ref()).await {
        Ok(_) => {
            eprintln!("User successfully inserted into database");
            return Ok(HttpResponse::Created().json(serde_json::json!({ 
                "message": "User registered successfully"
            })));
        }
        Err(e) => {
            eprintln!("Database insertion error: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Error registering user"
            })));
        }
    }
}

#[get("/users")]
async fn get_users(db: web::Data<DatabaseConnection>, req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];

                match AuthTokenClaims::validate_token(token) {
                    Ok(_) => {
                        let users = UserEntity::find()
                            .all(db.as_ref())
                            .await
                            .map_err(|_| actix_web::error::ErrorInternalServerError("Error fetching users"))?
                            .into_iter()
                            .map(|user| serde_json::json!({
                                "id": user.id,
                                "email": user.email,
                            }))
                            .collect::<Vec<_>>();

                        return Ok(HttpResponse::Ok().json(users));
                    },
                    Err(_) => {
                        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                            "error": "Invalid token"
                        })));
                    }
                }
            }
        }
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid token format"
        })));
    }

    Ok(HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Missing token"
    })))
}

//user profile API


use crate::entities::userprofile;



#[derive(Deserialize)]
pub struct NewUserProfile {
    pub user_id: i32,
    pub profile_photo: Option<String>,
    pub about: Option<String>,
    pub location: Option<String>,
    pub language: Option<String>,
    pub phone_number: String,
}

#[post("/user_profiles")]
async fn create_user_profile(
    new_profile: web::Json<NewUserProfile>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let new_profile_active = userprofile::ActiveModel {
        user_id: Set(new_profile.user_id),
        profile_photo: Set(new_profile.profile_photo.clone()),
        about: Set(new_profile.about.clone()),
        location: Set(new_profile.location.clone()),
        language: Set(new_profile.language.clone()),
        phone_number: Set(new_profile.phone_number.clone()),
        ..Default::default()
    };

    match new_profile_active.insert(db.as_ref()).await {
        Ok(_) => Ok(HttpResponse::Created().json(serde_json::json!({
            "message": "User profile created successfully"
        }))),
        Err(e) => {
            eprintln!("Database insertion error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Error creating user profile"
            })))
        }
    }
}

#[get("/user_profiles")]
async fn get_user_profiles(db: web::Data<DatabaseConnection>) -> Result<HttpResponse, Error> {
    match userprofile::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(profiles) => Ok(HttpResponse::Ok().json(profiles)),
        Err(e) => {
            eprintln!("Database query error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Error fetching user profiles"
            })))
        }
    }
}

#[put("/user_profiles/{user_id}")]
async fn update_user_profile(
    user_id: web::Path<i32>,
    updated_profile: web::Json<NewUserProfile>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let existing_profile = userprofile::Entity::find()
        .filter(userprofile::Column::UserId.eq(user_id.into_inner()))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            eprintln!("Database query error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if let Some(profile) = existing_profile {
        let mut profile_active: userprofile::ActiveModel = profile.into();

        profile_active.profile_photo = Set(updated_profile.profile_photo.clone());
        profile_active.about = Set(updated_profile.about.clone());
        profile_active.location = Set(updated_profile.location.clone());
        profile_active.language = Set(updated_profile.language.clone());
        profile_active.phone_number = Set(updated_profile.phone_number.clone());

        match profile_active.update(db.as_ref()).await {
            Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "User profile updated successfully"
            }))),
            Err(e) => {
                eprintln!("Database update error: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Error updating user profile"
                })))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "User profile not found"
        })))
    }
}

#[delete("/user_profiles/{user_id}")]
async fn delete_user_profile(
    user_id: web::Path<i32>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let deleted_count = userprofile::Entity::delete_many()
        .filter(userprofile::Column::UserId.eq(user_id.into_inner()))
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            eprintln!("Database deletion error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .rows_affected;

    if deleted_count > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "User profile deleted successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "User profile not found"
        })))
    }
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


//vehicleentity API


#[derive(Debug, Deserialize)]
pub struct CreateVehicle {
    pub driver_id: i32,
    pub vehicle_type: String,
    pub style: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub license_plate: String,
    pub passenger_capacity: i32,
    pub photo: String,
    pub base_fare: Option<f64>,
    pub per_minute_rate: Option<f64>,
    pub per_kilometer_rate: Option<f64>,
    pub status: String,
}

#[get("/vehicles")]
pub async fn get_all_vehicles(db: web::Data<DatabaseConnection>) -> impl Responder {
    match vehicleentity::Entity::find().all(db.get_ref()).await {
        Ok(vehicle_list) => HttpResponse::Ok().json(vehicle_list),
        Err(e) => {
            eprintln!("Failed to fetch vehicles: {:?}", e); // Log the error for debugging
            HttpResponse::InternalServerError().body(format!("Failed to fetch vehicles: {:?}", e))
        }
    }
}

#[get("/vehicles/{id}")]
pub async fn get_vehicle(db: web::Data<DatabaseConnection>, vehicle_id: web::Path<i32>) -> impl Responder {
    match vehicleentity::Entity::find_by_id(vehicle_id.into_inner()).one(db.get_ref()).await {
        Ok(Some(vehicle)) => HttpResponse::Ok().json(vehicle),
        Ok(None) => HttpResponse::NotFound().body("Vehicle not found"),
        Err(e) => {
            eprintln!("Failed to fetch vehicle: {:?}", e); // Log the error for debugging
            HttpResponse::InternalServerError().body(format!("Failed to fetch vehicle: {:?}", e))
        }
    }
}

#[post("/vehicles")]
pub async fn create_vehicle(
    db: web::Data<DatabaseConnection>,
    vehicle_data: web::Json<CreateVehicle>,
) -> impl Responder {
    eprintln!("Received vehicle data: {:?}", vehicle_data);

    // Check if a vehicle for the same driver already exists
    let existing_vehicle = vehicleentity::Entity::find()
        .filter(vehicleentity::Column::DriverId.eq(vehicle_data.driver_id))
        .one(db.get_ref())
        .await;

    match existing_vehicle {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().body("Vehicle already created for this driver"); 
        }
        Ok(None) => {} 
        Err(e) => {
            eprintln!("Error checking existing vehicle: {:?}", e); 
            return HttpResponse::InternalServerError().body(format!("Error checking existing vehicle: {:?}", e));
        }
    }

    // Create a new vehicle
    let new_vehicle = vehicleentity::ActiveModel {
        driver_id: Set(vehicle_data.driver_id),
        vehicle_type: Set(vehicle_data.vehicle_type.clone()),
        style: Set(vehicle_data.style.clone()),
        make: Set(vehicle_data.make.clone()),
        model: Set(vehicle_data.model.clone()),
        year: Set(vehicle_data.year),
        license_plate: Set(vehicle_data.license_plate.clone()),
        passenger_capacity: Set(vehicle_data.passenger_capacity),
        photo: Set(vehicle_data.photo.clone()),
        base_fare: Set(vehicle_data.base_fare), 
        per_minute_rate: Set(vehicle_data.per_minute_rate), 
        per_kilometer_rate: Set(vehicle_data.per_kilometer_rate), 
        status: Set(vehicle_data.status.clone()),
        ..Default::default() 
    };

    eprintln!("Inserting new vehicle: {:?}", new_vehicle);

    match new_vehicle.insert(db.get_ref()).await {
        Ok(vehicle) => HttpResponse::Created().json(serde_json::json!({
            "message": "Vehicle created successfully",
            "vehicle": vehicle
        })),
        Err(e) => {
            eprintln!("Failed to create vehicle: {:?}", e); 
            HttpResponse::InternalServerError().body(format!("Failed to create vehicle: {:?}", e))
        }
    }
}

#[delete("/vehicles/{id}")]
pub async fn delete_vehicle(db: web::Data<DatabaseConnection>, vehicle_id: web::Path<i32>) -> impl Responder {
    match vehicleentity::Entity::delete_by_id(vehicle_id.into_inner()).exec(db.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("Vehicle deleted successfully"),
        Err(e) => {
            eprintln!("Failed to delete vehicle: {:?}", e); 
            HttpResponse::InternalServerError().body(format!("Failed to delete vehicle: {:?}", e))
        }
    }
}


//ride entity API




#[derive(Debug, Deserialize)]
pub struct CreateRide {
    pub user_id: i32,
    pub driver_id: i32,
    pub vehicle_id: i32,
    pub ride_type: String,
    pub vehicle_type: String,
    pub pickup_location: String,
    pub pickup_lat: f64,
    pub pickup_lng: f64,
    pub dropoff_location: String,
    pub dropoff_lat: f64,
    pub dropoff_lng: f64,
    pub scheduled_time: Option<ChronoDateTime<Utc>>,
    pub start_time: Option<ChronoDateTime<Utc>>,
    pub end_time: Option<ChronoDateTime<Utc>>,
    pub status: String,
    pub distance_fare: Decimal,
    pub time_fare: Decimal,
    pub tip_amount: Option<Decimal>,
    pub total_amount: Decimal,
    pub rating: Option<i16>,
    pub review: Option<String>,
    pub cancel_reason: Option<String>,
    pub payment_status: String,
    pub payment_id: i32,
}

#[get("/rides")]
pub async fn get_all_rides(db: web::Data<DatabaseConnection>) -> impl Responder {
    match RideEntity::find().all(db.get_ref()).await {
        Ok(ride_list) => HttpResponse::Ok().json(ride_list),
        Err(e) => {
            eprintln!("Failed to fetch rides: {:?}", e); 
            HttpResponse::InternalServerError().body(format!("Failed to fetch rides: {:?}", e))
        }
    }
}

#[get("/rides/{id}")]
pub async fn get_ride(db: web::Data<DatabaseConnection>, ride_id: web::Path<i32>) -> impl Responder {
    match RideEntity::find_by_id(ride_id.into_inner()).one(db.get_ref()).await {
        Ok(Some(ride)) => HttpResponse::Ok().json(ride),
        Ok(None) => HttpResponse::NotFound().body("Ride not found"),
        Err(e) => {
            eprintln!("Failed to fetch ride: {:?}", e); 
            HttpResponse::InternalServerError().body(format!("Failed to fetch ride: {:?}", e))
        }
    }
}

#[post("/rides")]
pub async fn create_ride(
    db: web::Data<DatabaseConnection>,
    ride_data: web::Json<CreateRide>,
) -> impl Responder {
    // Create a new ride
    let new_ride = rideentity::ActiveModel {
        user_id: Set(ride_data.user_id),
        driver_id: Set(ride_data.driver_id),
        vehicle_id: Set(ride_data.vehicle_id),
        ride_type: Set(ride_data.ride_type.clone()),
        vehicle_type: Set(ride_data.vehicle_type.clone()),
        pickup_location: Set(ride_data.pickup_location.clone()),
        pickup_lat: Set(ride_data.pickup_lat),
        pickup_lng: Set(ride_data.pickup_lng),
        dropoff_location: Set(ride_data.dropoff_location.clone()),
        dropoff_lat: Set(ride_data.dropoff_lat),
        dropoff_lng: Set(ride_data.dropoff_lng),
        scheduled_time: Set(ride_data.scheduled_time.clone()),
        start_time: Set(ride_data.start_time.clone()),
        end_time: Set(ride_data.end_time.clone()),
        status: Set(ride_data.status.clone()),
        distance_fare: Set(ride_data.distance_fare),
        time_fare: Set(ride_data.time_fare),
        tip_amount: Set(ride_data.tip_amount),
        total_amount: Set(ride_data.total_amount),
        rating: Set(ride_data.rating),
        review: Set(ride_data.review.clone()),
        cancel_reason: Set(ride_data.cancel_reason.clone()),
        payment_status: Set(ride_data.payment_status.clone()),
        payment_id: Set(ride_data.payment_id),
        ..Default::default() 
    };

    match new_ride.insert(db.get_ref()).await {
        Ok(ride) => HttpResponse::Created().json(serde_json::json!({
            "message": "Ride created successfully",
            "ride": ride
        })),
        Err(e) => {
            eprintln!("Failed to create ride: {:?}", e); 
            HttpResponse::InternalServerError().body(format!("Failed to create ride: {:?}", e))
        }
    }
}

#[delete("/rides/{id}")]
pub async fn delete_ride(db: web::Data<DatabaseConnection>, ride_id: web::Path<i32>) -> impl Responder {
    match RideEntity::delete_by_id(ride_id.into_inner()).exec(db.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("Ride deleted successfully"),
        Err(e) => {
            eprintln!("Failed to delete ride: {:?}", e); 
            HttpResponse::InternalServerError().body(format!("Failed to delete ride: {:?}", e))
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_rides)
        .service(get_ride)
        .service(create_ride)
        .service(delete_ride);
}


//cities API 




#[derive(Debug, Deserialize)]
struct NewCity {
    pub name: String,
}

#[post("/cities")]
async fn add_cities(
    cities: web::Json<Vec<NewCity>>, 
    db: web::Data<DatabaseConnection>
) -> impl Responder {
    let insert_operations: Vec<_> = cities
        .iter()
        .map(|city| cities::ActiveModel {
            name: Set(city.name.clone()),
            ..Default::default()
        })
        .collect();

    match cities::Entity::insert_many(insert_operations).exec(db.get_ref()).await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({ "message": "Cities added successfully" })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": format!("Failed to add cities: {}", err) })),
    }
}


#[get("/cities")]
async fn get_cities(db: web::Data<DatabaseConnection>) -> impl Responder {
    match cities::Entity::find().all(db.get_ref()).await {
        Ok(cities) => HttpResponse::Ok().json(cities),
        Err(_) => HttpResponse::InternalServerError().json("Error fetching cities"),
    }
}

// settings API


#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSettings {
    pub user_id: i32,
    pub language: String,
    pub notifications_enabled: bool,
    pub dark_mode: bool,
    pub currency: String,
}


#[derive(Deserialize, Serialize)]
pub struct UpdateSettings {
    pub language: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub dark_mode: Option<bool>,
    pub currency: Option<String>,
}


#[post("/settings")]
async fn create_settings(
    db: web::Data<DatabaseConnection>,
    payload: web::Json<CreateSettings>,
) -> impl Responder {
    info!("Received request to create settings: {:?}", payload);

    let new_setting = settings::ActiveModel {
        user_id: Set(payload.user_id),
        language: Set(payload.language.clone()),
        notifications_enabled: Set(payload.notifications_enabled),
        dark_mode: Set(payload.dark_mode),
        currency: Set(payload.currency.clone()),
        ..Default::default()
    };

    match settings::Entity::insert(new_setting).exec(db.get_ref()).await {
        Ok(insert_result) => {
            // Fetch newly inserted settings
            match settings::Entity::find_by_id(insert_result.last_insert_id)
                .one(db.get_ref())
                .await
            {
                Ok(Some(inserted)) => {
                    info!("Successfully created settings with ID: {}", inserted.id);
                    HttpResponse::Created().json(inserted)
                }
                Ok(None) => {
                    error!("Inserted settings not found.");
                    HttpResponse::InternalServerError().json("Inserted settings not found.")
                }
                Err(e) => {
                    error!("Database fetch error: {:?}", e);
                    HttpResponse::InternalServerError().json(format!("Database error: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Database insert error: {:?}", e);
            HttpResponse::InternalServerError().json(format!("Database insert error: {}", e))
        }
    }
}


/// Get a settings entry by ID
#[get("/settings/{id}")]
async fn get_settings(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> impl Responder {
    let id = id.into_inner();
    info!("Fetching settings for ID: {}", id);

    match settings::Entity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(setting)) => HttpResponse::Ok().json(setting),
        Ok(None) => {
            info!("Settings not found for ID: {}", id);
            HttpResponse::NotFound().json(format!("No settings found for ID: {}", id))
        }
        Err(e) => {
            error!("Database error fetching settings: {:?}", e);
            HttpResponse::InternalServerError().json("Database error retrieving settings.")
        }
    }
}

/// Update a settings entry
#[put("/settings/{id}")]
async fn update_settings(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
    payload: web::Json<UpdateSettings>,
) -> impl Responder {
    let id = id.into_inner();
    info!("Updating settings for ID: {}", id);

    match settings::Entity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(setting)) => {
            let mut active_model: settings::ActiveModel = setting.into();

            if let Some(language) = &payload.language {
                active_model.language = Set(language.clone());
            }
            if let Some(notifications_enabled) = payload.notifications_enabled {
                active_model.notifications_enabled = Set(notifications_enabled);
            }
            if let Some(dark_mode) = payload.dark_mode {
                active_model.dark_mode = Set(dark_mode);
            }
            if let Some(currency) = &payload.currency {
                active_model.currency = Set(currency.clone());
            }

            match active_model.update(db.get_ref()).await {
                Ok(updated) => {
                    info!("Successfully updated settings for ID: {}", id);
                    HttpResponse::Ok().json(updated)
                }
                Err(e) => {
                    error!("Database error updating settings: {:?}", e);
                    HttpResponse::InternalServerError().json("Database error updating settings.")
                }
            }
        }
        Ok(None) => {
            info!("Settings not found for ID: {}", id);
            HttpResponse::NotFound().json(format!("No settings found for ID: {}", id))
        }
        Err(e) => {
            error!("Database error fetching settings for update: {:?}", e);
            HttpResponse::InternalServerError().json("Database error retrieving settings for update.")
        }
    }
}

/// Delete a settings entry
#[delete("/settings/{id}")]
async fn delete_settings(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> impl Responder {
    let id = id.into_inner();
    info!("Deleting settings for ID: {}", id);

    match settings::Entity::delete_by_id(id).exec(db.get_ref()).await {
        Ok(delete_result) if delete_result.rows_affected > 0 => {
            info!("Successfully deleted settings for ID: {}", id);
            HttpResponse::Ok().json(format!("Deleted settings with ID: {}", id))
        }
        Ok(_) => {
            info!("No settings found for ID: {}", id);
            HttpResponse::NotFound().json(format!("No settings found for ID: {}", id))
        }
        Err(e) => {
            error!("Database error deleting settings: {:?}", e);
            HttpResponse::InternalServerError().json("Database error deleting settings.")
        }
    }
}


//helpsupport API

use crate::entities::helpsupport::{Entity as HelpSupportEntity, ActiveModel};




/// Create a new support ticket
#[post("/tickets")]
async fn create_ticket(
    db: web::Data<DatabaseConnection>,
    payload: web::Json<NewTicketRequest>,
) -> impl Responder {
    let new_ticket = entities::helpsupport::ActiveModel {
        user_id: Set(payload.user_id),
        subject: Set(payload.subject.clone()),
        description: Set(payload.description.clone()),
        status: Set(payload.status.clone()),
        priority: Set(payload.priority.clone()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };

    match new_ticket.insert(db.get_ref()).await {
        Ok(ticket) => {
            let response = serde_json::json!({
                "message": "Support ticket created successfully",
                "ticket": ticket  // Includes the created ticket details
            });
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            error!("Failed to create support ticket: {}", e);
            let error_response = serde_json::json!({
                "error": "Failed to create support ticket",
                "details": e.to_string()
            });
            HttpResponse::InternalServerError().json(error_response)
        }
    }
}


/// Get all support tickets
#[get("/tickets")]
async fn get_tickets(db: web::Data<DatabaseConnection>) -> impl Responder {
    match HelpSupportEntity::find().all(db.get_ref()).await {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(e) => {
            error!("Failed to fetch support tickets: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch support tickets")
        }
    }
}

/// Get a support ticket by ID
#[get("/tickets/{id}")]
async fn get_ticket(db: web::Data<DatabaseConnection>, id: web::Path<i32>) -> impl Responder {
    match HelpSupportEntity::find_by_id(id.into_inner()).one(db.get_ref()).await {
        Ok(Some(ticket)) => HttpResponse::Ok().json(ticket),
        Ok(None) => HttpResponse::NotFound().body("Support ticket not found"),
        Err(e) => {
            error!("Failed to fetch support ticket: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch support ticket")
        }
    }
}

/// Update a support ticket by ID
#[put("/tickets/{id}")]
async fn update_ticket(
    db: web::Data<DatabaseConnection>, 
    id: web::Path<i32>, 
    ticket: web::Json<NewTicketRequest>  // Use correct struct
) -> impl Responder {
    let ticket_id = id.into_inner();

    match entities::helpsupport::Entity::find_by_id(ticket_id).one(db.get_ref()).await {
        Ok(Some(existing_ticket)) => {
            let mut active_ticket: entities::helpsupport::ActiveModel = existing_ticket.into();
            active_ticket.user_id = Set(ticket.user_id);
            active_ticket.subject = Set(ticket.subject.clone());
            active_ticket.description = Set(ticket.description.clone());
            active_ticket.status = Set(ticket.status.clone());
            active_ticket.priority = Set(ticket.priority.clone());
            active_ticket.updated_at = Set(chrono::Utc::now().naive_utc()); // Auto update timestamp

            match active_ticket.update(db.get_ref()).await {
                Ok(updated_ticket) => {
                    let response = serde_json::json!({
                        "message": "Support ticket updated successfully",
                        "ticket": updated_ticket
                    });
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    error!("Failed to update support ticket: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to update support ticket",
                        "details": e.to_string()
                    }))
                }
            }
        },
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Support ticket not found"})),
        Err(e) => {
            error!("Failed to fetch support ticket: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch support ticket",
                "details": e.to_string()
            }))
        }
    }
}

/// Delete a support ticket by ID
#[delete("/tickets/{id}")]
async fn delete_ticket(db: web::Data<DatabaseConnection>, id: web::Path<i32>) -> impl Responder {
    let ticket_id = id.into_inner();

    match HelpSupportEntity::find_by_id(ticket_id).one(db.get_ref()).await {
        Ok(Some(ticket)) => {
            let active_ticket: ActiveModel = ticket.into();

            match active_ticket.delete(db.get_ref()).await {
                Ok(_) => HttpResponse::Ok().body("Support ticket deleted successfully"),
                Err(e) => {
                    error!("Failed to delete support ticket: {}", e);
                    HttpResponse::InternalServerError().body("Failed to delete support ticket")
                }
            }
        },
        Ok(None) => HttpResponse::NotFound().body("Support ticket not found"),
        Err(e) => {
            error!("Failed to fetch support ticket: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch support ticket")
        }
    }
}

//recentlocation API


use crate::entities::recentlocation;



#[derive(Deserialize)]
pub struct NewRecentLocationRequest {
    pub user_id: i32,
    pub location_name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub frequency: i32,
}

#[post("/recent-locations")]
async fn add_recent_location(
    db: web::Data<DatabaseConnection>,
    payload: web::Json<NewRecentLocationRequest>,  
) -> impl Responder {
    let now = Utc::now().naive_utc(); 

    let new_location = recentlocation::ActiveModel {
        user_id: Set(payload.user_id),
        location_name: Set(payload.location_name.clone()),
        address: Set(payload.address.clone()),
        lat: Set(payload.lat),
        lng: Set(payload.lng),
        frequency: Set(payload.frequency),
        last_used: Set(now),  
        created_at: Set(now), 
        updated_at: Set(now), 
        ..Default::default()  
    };

    match new_location.insert(db.get_ref()).await {
        Ok(location) => {
            let response = serde_json::json!({
                "message": "Recent location added successfully",
                "location": location
            });
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            error!("Failed to add recent location: {}", e);
            let error_response = serde_json::json!({
                "error": "Failed to add recent location",
                "details": e.to_string()
            });
            HttpResponse::InternalServerError().json(error_response)
        }
    }
}


#[get("/recent-locations/{user_id}")]
async fn get_recent_locations(db: web::Data<DatabaseConnection>, user_id: web::Path<i32>) -> impl Responder {
    let user_id = user_id.into_inner();
    match recentlocation::Entity::find()
        .filter(recentlocation::Column::UserId.eq(user_id))
        .all(db.get_ref())
        .await
    {
        Ok(locations) if !locations.is_empty() => HttpResponse::Ok().json(locations),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({ "error": "No recent locations found" })),
        Err(e) => {
            error!("Failed to fetch recent locations: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch recent locations",
                "details": e.to_string()
            }))
        }
    }
}

// payment API



// #[derive(Serialize, Deserialize)]
// pub struct PaymentRequest {
//     pub user_id: i32,
//     pub payment_type: String,
//     pub card_number: Option<String>,
//     pub card_holder: Option<String>,
//     pub expiry_month: Option<i16>,
//     pub expiry_year: Option<i16>,
//     pub card_type: Option<String>,
//     pub is_default: bool,
//     pub paypal_email: Option<String>,
// }

// /// GET API - Fetch all payments
// #[get("/payments")]
// async fn get_payments(db: web::Data<DatabaseConnection>) -> impl Responder {
//     match PaymentEntity::find().all(db.get_ref()).await {
//         Ok(payments) => HttpResponse::Ok().json(payments),
//         Err(e) => {
//             eprintln!("Error fetching payments: {:?}", e);
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": format!("Failed to fetch payments: {:?}", e)
//             }))
//         }
//     }
// }

// /// POST API - Create a new payment
// #[post("/payments")]
// async fn create_payment(
//     db: web::Data<DatabaseConnection>,
//     payment_data: web::Json<PaymentRequest>,
// ) -> impl Responder {
//     let new_payment = PaymentActiveModel {
//         user_id: Set(payment_data.user_id),
//         payment_type: Set(payment_data.payment_type.clone()),
//         card_number: Set(payment_data.card_number.clone()),
//         card_holder: Set(payment_data.card_holder.clone()),
//         expiry_month: Set(payment_data.expiry_month),
//         expiry_year: Set(payment_data.expiry_year),
//         card_type: Set(payment_data.card_type.clone()),
//         is_default: Set(payment_data.is_default),
//         paypal_email: Set(payment_data.paypal_email.clone()),
//         created_at: Set(Some(Utc::now())),
//         updated_at: Set(Some(Utc::now())),
//         ..Default::default()
//     };

//     match new_payment.insert(db.get_ref()).await {
//         Ok(inserted) => HttpResponse::Created().json(serde_json::json!({
//             "message": "Payment created successfully",
//             "payment": inserted
//         })),
//         Err(e) => {
//             eprintln!("Error inserting payment: {:?}", e);
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": format!("Failed to create payment: {:?}", e)
//             }))
//         }
//     }
// }

// /// PUT API - Update a payment
// #[put("/payments/{id}")]
// async fn update_payment(
//     db: web::Data<DatabaseConnection>,
//     payment_id: web::Path<i32>,
//     payment_data: web::Json<PaymentRequest>,
// ) -> impl Responder {
//     let id = payment_id.into_inner();
    
//     match PaymentEntity::find_by_id(id).one(db.get_ref()).await {
//         Ok(Some(existing_payment)) => {
//             let mut updated_payment: PaymentActiveModel = existing_payment.into();
//             updated_payment.payment_type = Set(payment_data.payment_type.clone());
//             updated_payment.card_number = Set(payment_data.card_number.clone());
//             updated_payment.card_holder = Set(payment_data.card_holder.clone());
//             updated_payment.expiry_month = Set(payment_data.expiry_month);
//             updated_payment.expiry_year = Set(payment_data.expiry_year);
//             updated_payment.card_type = Set(payment_data.card_type.clone());
//             updated_payment.is_default = Set(payment_data.is_default);
//             updated_payment.paypal_email = Set(payment_data.paypal_email.clone());
//             updated_payment.updated_at = Set(Some(Utc::now()));

//             match updated_payment.update(db.get_ref()).await {
//                 Ok(updated) => HttpResponse::Ok().json(serde_json::json!({
//                     "message": "Payment updated successfully",
//                     "payment": updated
//                 })),
//                 Err(e) => {
//                     eprintln!("Error updating payment: {:?}", e);
//                     HttpResponse::InternalServerError().json(serde_json::json!({
//                         "error": format!("Failed to update payment: {:?}", e)
//                     }))
//                 }
//             }
//         }
//         Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
//             "error": "Payment not found. Please check the payment ID."
//         })),
//         Err(e) => {
//             eprintln!("Database error while updating payment: {:?}", e);
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": format!("Database error: {:?}", e)
//             }))
//         }
//     }
// }

// /// DELETE API - Delete a payment
// #[delete("/payments/{id}")]
// async fn delete_payment(
//     db: web::Data<DatabaseConnection>,
//     payment_id: web::Path<i32>,
// ) -> impl Responder {
//     let id = payment_id.into_inner();

//     match PaymentEntity::find_by_id(id).one(db.get_ref()).await {
//         Ok(Some(existing_payment)) => {
//             let active_payment: PaymentActiveModel = existing_payment.into();

//             match active_payment.delete(db.get_ref()).await {
//                 Ok(_) => HttpResponse::Ok().json(serde_json::json!({
//                     "message": format!("Payment with ID {} deleted successfully", id)
//                 })),
//                 Err(e) => {
//                     eprintln!("Error deleting payment: {:?}", e);
//                     HttpResponse::InternalServerError().json(serde_json::json!({
//                         "error": format!("Failed to delete payment: {:?}", e)
//                     }))
//                 }
//             }
//         }
//         Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
//             "error": "Payment not found. Please check the payment ID."
//         })),
//         Err(e) => {
//             eprintln!("Database error while deleting payment: {:?}", e);
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": format!("Database error: {:?}", e)
//             }))
//         }
//     }
// }

// /// Initialize payment routes
// pub fn init(cfg: &mut web::ServiceConfig) {
//     cfg.service(get_payments)
//        .service(create_payment)
//        .service(update_payment)
//        .service(delete_payment);
// }

