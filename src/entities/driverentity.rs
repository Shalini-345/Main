use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "drivers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub photo: String,
    pub rating: f32,
    pub total_rides: i32,
    pub about_me: String,
    pub from_location: String,
    pub languages: Vec<String>, 
    pub is_pilot: bool,
    pub license_number: String,
    pub verification_status: String,
    pub current_lat: f64,
    pub current_lng: f64,
    pub availability_status: String,
    #[sea_orm(default_value = "now()")]
    pub created_at: Option<chrono::NaiveDateTime>,  
    #[sea_orm(default_value = "now()", on_update = "now()")]
    pub updated_at: Option<chrono::NaiveDateTime>,  

}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
