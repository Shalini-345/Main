use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "users")] // Ensure table name matches your actual DB table
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,  
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

// ✅ Ensure Related implementation for seamless queries
impl Related<crate::entities::cities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::City.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
