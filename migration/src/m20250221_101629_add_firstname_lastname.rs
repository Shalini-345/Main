use sea_orm_migration::prelude::*;
use sea_orm::DbErr;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Check if first_name column exists, if not, add it
        if !manager.has_column(Users::Table.as_ref(), Users::FirstName.as_ref()).await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .add_column(ColumnDef::new(Users::FirstName).string().not_null())
                        .to_owned(),
                )
                .await?;
        }

        // Check if last_name column exists, if not, add it
        if !manager.has_column(Users::Table.as_ref(), Users::LastName.as_ref()).await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .add_column(ColumnDef::new(Users::LastName).string().not_null())
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::FirstName)
                    .drop_column(Users::LastName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    FirstName,
    LastName,
}

impl AsRef<str> for Users {
    fn as_ref(&self) -> &str {
        match self {
            Users::Table => "users",
            Users::FirstName => "first_name",
            Users::LastName => "last_name",
        }
    }
}
