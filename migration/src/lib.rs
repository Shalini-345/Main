pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250213_073143_create_drivers_table;
mod m20250213_164001_create_users_table;
mod m20250215_092124_create_payment_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250213_073143_create_drivers_table::Migration),
            Box::new(m20250213_164001_create_users_table::Migration),
            Box::new(m20250215_092124_create_payment_table::Migration),
        ]
    }
}
