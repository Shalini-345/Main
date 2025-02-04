use crate::entities::payment::Entity as PaymentEntity;
use crate::entities::userentity::Entity as UserEntity;
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveRelation, EnumIter};
use chrono::{DateTime as ChronoDateTime, Utc};
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ride")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
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
    pub created_at: ChronoDateTime<Utc>,
    pub updated_at: ChronoDateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "PaymentEntity", from = "Column::PaymentId", to = "super::payment::Column::Id")]
    Payment,  

    #[sea_orm(belongs_to = "UserEntity", from = "Column::UserId", to = "super::userentity::Column::Id")]
    User,  
}


impl Related<PaymentEntity> for Entity {
    fn to() -> RelationDef {
        Relation::Payment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
