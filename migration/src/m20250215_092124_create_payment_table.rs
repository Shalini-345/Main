use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Payment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payment::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payment::UserId).integer().not_null())
                    .col(ColumnDef::new(Payment::PaymentType).string().not_null())
                    .col(ColumnDef::new(Payment::CardNumber).string().null())
                    .col(ColumnDef::new(Payment::CardHolder).string().null())
                    .col(ColumnDef::new(Payment::ExpiryMonth).integer().null())
                    .col(ColumnDef::new(Payment::ExpiryYear).integer().null())
                    .col(ColumnDef::new(Payment::CardType).string().null())
                    .col(ColumnDef::new(Payment::IsDefault).boolean().not_null().default(false))
                    .col(ColumnDef::new(Payment::PaypalEmail).string().null())
                    .col(
                        ColumnDef::new(Payment::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Payment::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payment-user")
                            .from(Payment::Table, Payment::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Payment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Payment {
    Table,
    Id,
    UserId,
    PaymentType,
    CardNumber,
    CardHolder,
    ExpiryMonth,
    ExpiryYear,
    CardType,
    IsDefault,
    PaypalEmail,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    #[sea_orm(iden = "users")]  

    Table,
    Id,
}
