use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Settings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Settings::UserId).integer().not_null())
                    .col(ColumnDef::new(Settings::Language).string().not_null())
                    .col(ColumnDef::new(Settings::NotificationsEnabled).boolean().not_null())
                    .col(ColumnDef::new(Settings::DarkMode).boolean().not_null())
                    .col(ColumnDef::new(Settings::Currency).string().not_null())
                    .col(
                        ColumnDef::new(Settings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Settings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Settings {
    Table,
    Id,
    UserId,
    Language,
    NotificationsEnabled,
    DarkMode,
    Currency,
    CreatedAt,
    UpdatedAt,
}
