use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rides::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Rides::Id).integer().not_null().primary_key().auto_increment())
                    .col(ColumnDef::new(Rides::UserId).integer().not_null())
                    .col(ColumnDef::new(Rides::DriverId).integer().not_null())
                    .col(ColumnDef::new(Rides::VehicleId).integer().not_null())
                    .col(ColumnDef::new(Rides::RideType).string().not_null())
                    .col(ColumnDef::new(Rides::VehicleType).string().not_null())
                    .col(ColumnDef::new(Rides::PickupLocation).string().not_null())
                    .col(ColumnDef::new(Rides::PickupLat).double().not_null())
                    .col(ColumnDef::new(Rides::PickupLng).double().not_null())
                    .col(ColumnDef::new(Rides::DropoffLocation).string().not_null())
                    .col(ColumnDef::new(Rides::DropoffLat).double().not_null())
                    .col(ColumnDef::new(Rides::DropoffLng).double().not_null())
                    .col(ColumnDef::new(Rides::ScheduledTime).timestamp().null())
                    .col(ColumnDef::new(Rides::StartTime).timestamp().null())
                    .col(ColumnDef::new(Rides::EndTime).timestamp().null())
                    .col(ColumnDef::new(Rides::Status).string().not_null())
                    .col(ColumnDef::new(Rides::DistanceFare).decimal().not_null())
                    .col(ColumnDef::new(Rides::TimeFare).decimal().not_null())
                    .col(ColumnDef::new(Rides::TipAmount).decimal().null())
                    .col(ColumnDef::new(Rides::TotalAmount).decimal().not_null())
                    .col(ColumnDef::new(Rides::Rating).integer().null())
                    .col(ColumnDef::new(Rides::Review).string().null())
                    .col(ColumnDef::new(Rides::CancelReason).string().null())
                    .col(ColumnDef::new(Rides::PaymentStatus).string().not_null())
                    .col(ColumnDef::new(Rides::PaymentId).integer().not_null())
                    .col(ColumnDef::new(Rides::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Rides::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Rides::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Rides {
    Table,
    Id,
    UserId,
    DriverId,
    VehicleId,
    RideType,
    VehicleType,
    PickupLocation,
    PickupLat,
    PickupLng,
    DropoffLocation,
    DropoffLat,
    DropoffLng,
    ScheduledTime,
    StartTime,
    EndTime,
    Status,
    DistanceFare,
    TimeFare,
    TipAmount,
    TotalAmount,
    Rating,
    Review,
    CancelReason,
    PaymentStatus,
    PaymentId,
    CreatedAt,
    UpdatedAt,
}
