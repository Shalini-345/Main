use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
   
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[derive(Default)]
pub struct ActiveModel {
    pub id: ActiveValue<i32>,
    pub username: ActiveValue<String>,
    pub email: ActiveValue<String>,
    pub password: ActiveValue<String>,
    
}
