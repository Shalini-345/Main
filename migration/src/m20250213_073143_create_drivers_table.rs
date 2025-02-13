use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Drivers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Drivers::Id).integer().not_null().primary_key().auto_increment())
                    .col(ColumnDef::new(Drivers::FirstName).string().not_null())
                    .col(ColumnDef::new(Drivers::LastName).string().not_null())
                    .col(ColumnDef::new(Drivers::Email).string().unique_key().not_null())
                    .col(ColumnDef::new(Drivers::Phone).string().not_null())
                    .col(ColumnDef::new(Drivers::Photo).string().not_null())
                    .col(ColumnDef::new(Drivers::Rating).float().not_null().default(0.0))
                    .col(ColumnDef::new(Drivers::TotalRides).integer().not_null().default(0))
                    .col(ColumnDef::new(Drivers::AboutMe).string().not_null())
                    .col(ColumnDef::new(Drivers::FromLocation).string().not_null())
                    .col(ColumnDef::new(Drivers::Languages).string().not_null())
                    .col(ColumnDef::new(Drivers::IsPilot).boolean().not_null().default(false))
                    .col(ColumnDef::new(Drivers::LicenseNumber).string().not_null())
                    .col(ColumnDef::new(Drivers::VerificationStatus).string().not_null().default("pending"))
                    .col(ColumnDef::new(Drivers::CurrentLat).double().not_null().default(0.0))
                    .col(ColumnDef::new(Drivers::CurrentLng).double().not_null().default(0.0))
                    .col(ColumnDef::new(Drivers::AvailabilityStatus).string().not_null().default("unavailable"))
                    .col(ColumnDef::new(Drivers::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Drivers::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Drivers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Drivers {
    Table,
    Id,
    FirstName,
    LastName,
    Email,
    Phone,
    Photo,
    Rating,
    TotalRides,
    AboutMe,
    FromLocation,
    Languages,
    IsPilot,
    LicenseNumber,
    VerificationStatus,
    CurrentLat,
    CurrentLng,
    AvailabilityStatus,
    CreatedAt,
    UpdatedAt,
}
