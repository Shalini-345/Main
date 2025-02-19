use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id)) // Primary Key, auto-increment
                    .col(string(User::Email).not_null().unique_key()) // Email column (must be unique)
                    .col(string(User::Password).not_null()) // Password column
                    .col(string(User::City).not_null()) // City column
                    .col(string(User::PhoneNumber).not_null()) // Phone number column
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Password,
    City,
    PhoneNumber,
}
