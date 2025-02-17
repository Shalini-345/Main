use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Vehicles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Vehicles::Id).integer().not_null().primary_key().auto_increment())
                    .col(ColumnDef::new(Vehicles::DriverId).integer().not_null())
                    .col(ColumnDef::new(Vehicles::VehicleType).string().not_null())
                    .col(ColumnDef::new(Vehicles::Style).string().not_null())
                    .col(ColumnDef::new(Vehicles::Make).string().not_null())
                    .col(ColumnDef::new(Vehicles::Model).string().not_null())
                    .col(ColumnDef::new(Vehicles::Year).integer().not_null())
                    .col(ColumnDef::new(Vehicles::LicensePlate).string().not_null())
                    .col(ColumnDef::new(Vehicles::PassengerCapacity).integer().not_null())
                    .col(ColumnDef::new(Vehicles::Photo).string().not_null())
                    .col(ColumnDef::new(Vehicles::BaseFare).decimal().not_null())
                    .col(ColumnDef::new(Vehicles::PerMinuteRate).decimal().not_null())
                    .col(ColumnDef::new(Vehicles::PerKilometerRate).decimal().not_null())
                    .col(ColumnDef::new(Vehicles::Status).string().not_null())
                    .col(ColumnDef::new(Vehicles::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Vehicles::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vehicles::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Vehicles {
    Table,
    Id,
    DriverId,
    VehicleType,
    Style,
    Make,
    Model,
    Year,
    LicensePlate,
    PassengerCapacity,
    Photo,
    BaseFare,
    PerMinuteRate,
    PerKilometerRate,
    Status,
    CreatedAt,
    UpdatedAt,
}
