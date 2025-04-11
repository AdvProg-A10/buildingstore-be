#[macro_use] extern crate rocket;
use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx::{self, Row};
use rocket::serde::{Deserialize, Serialize};
use dotenvy::dotenv;

mod manajemen_produk;
use manajemen_produk::{daftar_produk, detail_produk, tambah_produk, update_produk, hapus_produk, filter_produk_by_kategori};

#[derive(Database)]
#[database("buildingstore")]
pub struct BuildingStoreDB(sqlx::PgPool);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/db")]
async fn test_db(mut db: Connection<BuildingStoreDB>) -> Option<String> {
    let row = sqlx::query("SELECT * FROM users LIMIT 1")
        .fetch_one(&mut **db)
        .await
        .ok()?;

    let id: i64 = row.get("id");
    let email: String = row.get("email");

    Some(format!("Hello, {}! Your ID is {}.", email, id))
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build()
        .manage(reqwest::Client::builder().build().unwrap())
        .attach(BuildingStoreDB::init())
        .mount("/", routes![index, test_db])
        .mount("/api", routes![
            daftar_produk,
            detail_produk,
            tambah_produk,
            update_produk,
            hapus_produk,
            filter_produk_by_kategori
        ])
}