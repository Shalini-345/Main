
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
    pub year: i16,
    pub license_plate: String,
    pub passenger_capacity: i16,
    pub photo: String,
    pub base_fare: Decimal,
    pub per_minute_rate: Decimal,
    pub per_kilometer_rate: Decimal,
    pub status: String, 
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}