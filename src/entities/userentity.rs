use sea_orm::entity::prelude::*;
use sea_orm::{DeriveRelation, EntityTrait};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub username: String,
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub city: String,
    pub state: String,
    pub profile_photo: Option<String>,
    pub remember_token: Option<String>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires: Option<DateTime>,
    pub about_me: Option<String>,
    pub languages: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
