
use sea_orm::DeriveRelation; 
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel,Serialize, Deserialize )]
#[sea_orm(table_name = "vehicles")]
 

pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub driver_id: i32,
    pub vehicle_type: String, 
    pub style: String, 
    pub make: String,
    pub model: String,
    #[sea_orm(column_type = "Integer")]
    pub year: i32,
    pub license_plate: String,
    pub passenger_capacity: i16,
    pub photo: String,
    pub base_fare: f64,
    pub per_minute_rate:f64,
    pub per_kilometer_rate: f64,
    pub status: String, 
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}