use rocket_db_pools::Database;

#[derive(Database)]
#[database("buildingstore")]
pub struct BuildingStoreDB(sqlx::PgPool);