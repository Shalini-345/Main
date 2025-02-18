use sea_orm::entity::prelude::*;
use sea_orm::DeriveRelation;
use serde::{Serialize, Deserialize}; 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DeriveEntityModel)] 
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32, 
    pub username: String,
    pub email: String,
    pub password: String,
    pub city: String,
    pub phone_number: String,


}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
