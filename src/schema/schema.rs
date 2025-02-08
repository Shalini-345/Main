use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
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

#[derive(Default)]
pub struct ActiveModel {
    pub id: ActiveValue<i32>,
    pub username: ActiveValue<String>,
    pub email: ActiveValue<String>,
    pub password: ActiveValue<String>,
    pub first_name: ActiveValue<Option<String>>,
    pub last_name: ActiveValue<Option<String>>,
    pub phone: ActiveValue<Option<String>>,
    pub city: ActiveValue<Option<String>>,
    pub state: ActiveValue<Option<String>>,
    pub profile_photo: ActiveValue<Option<String>>,
    pub remember_token: ActiveValue<Option<String>>,
    pub password_reset_token: ActiveValue<Option<String>>,
    pub password_reset_expires: ActiveValue<Option<DateTime>>,
    pub about_me: ActiveValue<Option<String>>,
    pub languages: ActiveValue<Option<String>>,
    pub created_at: ActiveValue<DateTime>,
    pub updated_at: ActiveValue<DateTime>,
}
