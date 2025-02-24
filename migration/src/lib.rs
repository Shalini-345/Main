pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250213_073143_create_drivers_table;
mod m20250213_164001_create_users_table;
mod m20250215_092124_create_payment_table;
mod m20250217_053312_create_vehicles_table;
mod m20250218_060218_create_ride_table;
mod m20250219_053115_create_cities_table;
mod m20250220_065436_create_settings_table;
mod m20250221_071147_create_support_tickets;
mod m20250221_101629_add_firstname_lastname;
mod m20250224_111441_create_user_profiles;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250213_073143_create_drivers_table::Migration),
            Box::new(m20250213_164001_create_users_table::Migration),
            Box::new(m20250215_092124_create_payment_table::Migration),
            Box::new(m20250217_053312_create_vehicles_table::Migration),
            Box::new(m20250218_060218_create_ride_table::Migration),
            Box::new(m20250219_053115_create_cities_table::Migration),
            Box::new(m20250220_065436_create_settings_table::Migration),
            Box::new(m20250221_071147_create_support_tickets::Migration),
            Box::new(m20250221_101629_add_firstname_lastname::Migration),
            Box::new(m20250224_111441_create_user_profiles::Migration),
        ]
    }
}
