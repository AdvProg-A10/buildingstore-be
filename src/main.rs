#[macro_use] extern crate rocket;
use rocket_db_pools::Database;
use rocket_db_pools::sqlx::{self, Row};
use buildingstore_be::BuildingStoreDB;
use dotenvy::dotenv;
use sqlx::any::install_default_drivers;
use rocket::State;
use sqlx::{Any, Pool};

pub mod auth;
pub mod manajemen_produk;
pub mod manajemen_pelanggan;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/db")]
async fn test_db(db: &State<Pool<Any>>) -> Option<String> {
    let mut db_conn = db.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM users LIMIT 1")
        .fetch_one(&mut *db_conn)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let email: String = row.get("username");

    Some(format!("Hello, {}! Your ID is {}.", email, id))
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    install_default_drivers();
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = sqlx::AnyPool::connect(&database_url).await.unwrap();
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    rocket::build()
        .manage(reqwest::Client::builder().build().unwrap())
        .manage(db_pool)
        .attach(BuildingStoreDB::init())
        .attach(auth::controller::route_stage())
        .attach(manajemen_pelanggan::controller::route_stage())
        .mount("/", routes![index, test_db])
}