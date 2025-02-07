use sea_orm_migration::prelude::*;
use migration::{Migrator, MigratorTrait};
 HEAD

 e10568085eb9bf2ba6305e4e0e3c7c878e230c89

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
