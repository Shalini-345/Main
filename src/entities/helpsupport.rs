use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "support_tickets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,  
    pub user_id: i32,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Serialize, Deserialize)]
pub struct NewTicketRequest {
    pub user_id: i32,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
