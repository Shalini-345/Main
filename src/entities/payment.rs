use crate::entities::userentity::Entity as UserEntity;
use crate::entities::rideentity::Entity as RideEntity; 
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveRelation, EnumIter};
use chrono::{DateTime as ChronoDateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "payment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub payment_type: String,
    pub card_number: Option<String>,
    pub card_holder: Option<String>,
    pub expiry_month: Option<i16>,
    pub expiry_year: Option<i16>,
    pub card_type: Option<String>,
    pub is_default: bool,
    pub paypal_email: Option<String>,
    pub created_at: Option<ChronoDateTime<Utc>>,
    pub updated_at: Option<ChronoDateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "UserEntity", from = "Column::UserId", to = "super::userentity::Column::Id")]
    User,  

    #[sea_orm(has_many = "RideEntity")]  
    Ride,  
}

impl ActiveModelBehavior for ActiveModel {}
