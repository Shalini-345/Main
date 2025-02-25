use sea_orm::entity::prelude::*;
use sea_orm::DeriveRelation; 
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel , Serialize ,Deserialize)]
#[sea_orm(table_name = "recent_locations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub location_name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub frequency: i32,
    pub last_used: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
