use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserProfiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserProfiles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserProfiles::UserId)
                            .integer()
                            .not_null()
                            .unique_key(), 
                    )
                    .col(
                        ColumnDef::new(UserProfiles::ProfilePhoto)
                            .string()
                            .null(), 
                    )
                    .col(
                        ColumnDef::new(UserProfiles::About)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(UserProfiles::Location)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserProfiles::Language)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserProfiles::PhoneNumber)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserProfiles::Table, UserProfiles::UserId)
                            .to(Users::Table, Users::Id) // Assuming a users table exists
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserProfiles::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum UserProfiles {
    Table,
    Id,
    UserId,
    ProfilePhoto,
    About,
    Location,
    Language,
    PhoneNumber,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
