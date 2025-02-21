use sea_orm::entity::prelude::*;
use sea_orm::DeriveRelation;
use serde::{Serialize, Deserialize}; 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DeriveEntityModel)] 
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,  // ✅ `id` is now the correct primary key
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub city: i32,  // ✅ Correct foreign key reference
    pub phone_number: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::entities::cities::Entity",
        from = "Column::City",
        to = "crate::entities::cities::Column::Id"
    )]
    City,  // ✅ Foreign key relation with the `cities` table
}

impl ActiveModelBehavior for ActiveModel {}
