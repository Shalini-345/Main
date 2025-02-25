use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RecentLocations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RecentLocations::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RecentLocations::UserId).integer().not_null())
                    .col(ColumnDef::new(RecentLocations::LocationName).string().not_null())
                    .col(ColumnDef::new(RecentLocations::Address).string().not_null())
                    .col(ColumnDef::new(RecentLocations::Lat).double().not_null())
                    .col(ColumnDef::new(RecentLocations::Lng).double().not_null())
                    .col(ColumnDef::new(RecentLocations::Frequency).integer().not_null())
                    .col(ColumnDef::new(RecentLocations::LastUsed).timestamp().not_null())
                    .col(ColumnDef::new(RecentLocations::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(RecentLocations::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RecentLocations::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum RecentLocations {
    Table,
    Id,
    UserId,
    LocationName,
    Address,
    Lat,
    Lng,
    Frequency,
    LastUsed,
    CreatedAt,
    UpdatedAt,
}
