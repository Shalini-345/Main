use sea_orm_migration::prelude::*;
use migration::{Migrator, MigratorTrait};
 

 

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
