use serde::{Serialize, Deserialize};
use diesel::{Queryable, Insertable};
use crate::schema::users;

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Deserialize, Serialize)] 
#[diesel(table_name = users)] 
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}
