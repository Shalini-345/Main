use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SupportTickets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SupportTickets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SupportTickets::UserId).integer().not_null())
                    .col(ColumnDef::new(SupportTickets::Subject).string().not_null())
                    .col(ColumnDef::new(SupportTickets::Description).text().not_null())
                    .col(ColumnDef::new(SupportTickets::Status).string().not_null())
                    .col(ColumnDef::new(SupportTickets::Priority).string().not_null())
                    .col(
                        ColumnDef::new(SupportTickets::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SupportTickets::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SupportTickets::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SupportTickets {
    Table,
    Id,
    UserId,
    Subject,
    Description,
    Status,
    Priority,
    CreatedAt,
    UpdatedAt,
}
