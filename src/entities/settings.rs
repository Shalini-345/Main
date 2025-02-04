use sea_orm::entity::prelude::*;
use sea_orm::DeriveRelation; 
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, )]
#[sea_orm(table_name = "settings")]
pub struct Model{
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub language: String,
    pub notifications_enabled: bool,
    pub dark_mode: bool,
    pub currency: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}