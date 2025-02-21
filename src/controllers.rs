use actix_web::{delete, get, post,put, web, HttpRequest, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait,  Set};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::auth::{generate_access_token,AuthTokenClaims};
use crate::entities::userentity::{self};
use crate::entities::{driverentity, vehicleentity};
use crate::db::establish_connection_pool;
use serde_json::json;
//use chrono::Utc;
//use crate::entities::payment::{ActiveModel as PaymentActiveModel, Entity as PaymentEntity};
use crate::entities::rideentity::{self, Entity as RideEntity};
use rust_decimal::Decimal;
use chrono::{DateTime as ChronoDateTime, Utc};
use crate::entities::settings::{self};
use log::{error, info};
use std::sync::Arc;
use crate::entities::helpsupport::{self, Entity as SupportTicket};
use sea_orm::ModelTrait; 



use crate::entities::cities; 

#[derive(Deserialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub city: i32,  
    pub phone_number: String,
}

// ✅ Email validation
fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[\w.-]+@[a-zA-Z\d.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

// ✅ Phone validation
fn validate_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

#[post("/users/register")]
pub async fn register_user(
    db: web::Data<DatabaseConnection>,
    new_user: web::Json<NewUser>,
) -> impl Responder { 
    // ✅ Validate email
    if !is_valid_email(&new_user.email) {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid email format"}));
    }

    // ✅ Validate phone number
    if !validate_phone(&new_user.phone_number) {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid phone number format"}));
    }

    // ✅ Check if email already exists
    if let Ok(Some(_)) = userentity::Entity::find()
    .filter(userentity::Column::Email.eq(new_user.email.clone()))
    .one(db.get_ref())  // ✅ FIXED: `db.get_ref()` instead of `&db`
    .await
{
    return HttpResponse::Conflict().json(json!({"error": "Email already exists"}));
}

    // ✅ Validate city ID exists
    if let Ok(None) = cities::Entity::find()
    .filter(cities::Column::Id.eq(new_user.city))
    .one(db.get_ref())  // ✅ FIXED: `db.get_ref()`
    .await
{
    return HttpResponse::BadRequest().json(json!({"error": "Invalid city ID"}));
}


    // ✅ Hash password
    let password_hash = match hash(&new_user.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(err) => {
            eprintln!("Error hashing password: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({"error": "Failed to hash password"}));
        }
    };

    // ✅ Insert new user
    let new_user_active_model = userentity::ActiveModel {
        first_name: Set(new_user.first_name.clone()),
        last_name: Set(new_user.last_name.clone()),
        email: Set(new_user.email.clone()),
        password: Set(password_hash),
        city: Set(new_user.city),
        phone_number: Set(new_user.phone_number.clone()),
        ..Default::default()
    };

    // ✅ Save to database
    if let Err(err) = userentity::Entity::insert(new_user_active_model)
    .exec(db.get_ref())  // ✅ FIXED: `db.get_ref()`
    .await
{
    eprintln!("Database error while inserting user: {:?}", err);
    return HttpResponse::InternalServerError().json(json!({"error": "Failed to register user"}));
}

    // ✅ Generate tokens
    let access_token = match generate_access_token(&new_user.email) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error generating access token: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate access token"
            }));
        } 
    };
    
    let refresh_token = match generate_access_token(&new_user.email) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error generating refresh token: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate refresh token"
            }));
        } 
    };
    
    HttpResponse::Created().json(json!({
        "message": "User registered successfully",
        "access_token": access_token,
        "refresh_token": refresh_token
    }))
}


#[get("/users")]
async fn get_users(db: web::Data<DatabaseConnection>, req: HttpRequest) -> impl Responder {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..]; 
                println!("Received Token: {:?}", token); 

                match AuthTokenClaims::validate_token(token) {
                    Ok(_) => {
                        match userentity::Entity::find().all(db.as_ref()).await {
                            Ok(users) => {
                                let user_list: Vec<_> = users.into_iter().map(|user| json!({
                                    "id": user.id,
                                    "email": user.email,
                                    "first_name": user.first_name,
                                    "last_name": user.last_name,
                                    "city": user.city,
                                    "phone_number": user.phone_number,
                                })).collect();
                                return HttpResponse::Ok().json(user_list);
                            },
                            Err(_) => return HttpResponse::InternalServerError().json(json!({
                                "error": "Failed to fetch users"
                            })),
                        }
                    },
                    Err(_) => return HttpResponse::Unauthorized().json(json!({
                        "error": "Invalid token"
                    })),
                }
            }
        }
        return HttpResponse::Unauthorized().json(json!({ "error": "Invalid token format" }));
    }

    HttpResponse::Unauthorized().json(json!({ "error": "Missing token" }))
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



#[derive(Debug, Deserialize, Serialize)]
struct TicketInput {
    user_id: i32,
    subject: String,
    description: String,
    status: String,
    priority: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TicketUpdateInput {
    subject: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

#[get("/tickets")]
async fn get_tickets(db: web::Data<Arc<DatabaseConnection>>) -> impl Responder {
    let db_conn = db.as_ref().as_ref();

    match SupportTicket::find().all(db_conn).await {
        Ok(tickets) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: if tickets.is_empty() { "No support tickets found.".to_string() } else { "Support tickets retrieved successfully.".to_string() },
            data: Some(tickets),
        }),
        Err(err) => {
            eprintln!("Error fetching tickets: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<Vec<helpsupport::Model>> {
                success: false,
                message: "Failed to fetch support tickets.".to_string(),
                data: None,
            })
        }
    }
}

#[post("/tickets")]
async fn create_ticket(db: web::Data<Arc<DatabaseConnection>>, new_ticket: web::Json<TicketInput>) -> impl Responder {
    let db_conn = db.as_ref().as_ref();

    let ticket = helpsupport::ActiveModel {
        user_id: Set(new_ticket.user_id),
        subject: Set(new_ticket.subject.clone()),
        description: Set(new_ticket.description.clone()),
        status: Set(new_ticket.status.clone()),
        priority: Set(new_ticket.priority.clone()),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    match ticket.insert(db_conn).await {
        Ok(ticket) => HttpResponse::Created().json(ApiResponse {
            success: true,
            message: "Support ticket created successfully.".to_string(),
            data: Some(ticket),
        }),
        Err(err) => {
            eprintln!("Error creating support ticket: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<helpsupport::Model> {
                success: false,
                message: format!("Failed to create support ticket: {:?}", err),
                data: None,
            })
        }
    }
}

#[put("/tickets/{id}")]
async fn update_ticket(
    db: web::Data<Arc<DatabaseConnection>>,
    ticket_id: web::Path<i32>,
    updated_ticket: web::Json<TicketUpdateInput>,
) -> impl Responder {
    let id = ticket_id.into_inner();
    let db_conn = db.as_ref().as_ref();

    match SupportTicket::find_by_id(id).one(db_conn).await {
        Ok(Some(ticket)) => {
            let mut active_model: helpsupport::ActiveModel = ticket.into();

            if let Some(subject) = &updated_ticket.subject {
                active_model.subject = Set(subject.clone());
            }
            if let Some(description) = &updated_ticket.description {
                active_model.description = Set(description.clone());
            }
            if let Some(status) = &updated_ticket.status {
                active_model.status = Set(status.clone());
            }
            if let Some(priority) = &updated_ticket.priority {
                active_model.priority = Set(priority.clone());
            }
            active_model.updated_at = Set(Utc::now().naive_utc());

            match active_model.update(db_conn).await {
                Ok(updated_ticket) => HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: "Support ticket updated successfully.".to_string(),
                    data: Some(updated_ticket),
                }),
                Err(err) => {
                    eprintln!("Error updating ticket: {:?}", err);
                    HttpResponse::InternalServerError().json(ApiResponse::<helpsupport::Model> {
                        success: false,
                        message: format!("Failed to update support ticket: {}", err),
                        data: None,
                    })
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Support ticket not found.".to_string(),
            data: None,
        }),
        Err(err) => {
            eprintln!("Error finding ticket: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: format!("Failed to find support ticket: {}", err),
                data: None,
            })
        }
    }
}

#[delete("/tickets/{id}")]
async fn delete_ticket(db: web::Data<Arc<DatabaseConnection>>, ticket_id: web::Path<i32>) -> impl Responder {
    let id = ticket_id.into_inner();
    let db_conn = db.as_ref().as_ref();

    match SupportTicket::find_by_id(id).one(db_conn).await {
        Ok(Some(ticket)) => {
            match ticket.delete(db_conn).await {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::<()> {
                    success: true,
                    message: "Support ticket deleted successfully.".to_string(),
                    data: None,
                }),
                Err(err) => {
                    eprintln!("Error deleting ticket: {:?}", err);
                    HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        message: "Failed to delete support ticket.".to_string(),
                        data: None,
                    })
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Support ticket not found.".to_string(),
            data: None,
        }),
        Err(err) => {
            eprintln!("Error finding ticket: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Failed to find support ticket.".to_string(),
                data: None,
            })
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

