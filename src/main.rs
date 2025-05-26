#[macro_use] extern crate rocket;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use buildingstore_be::{BuildingStoreDB};
use dotenvy::dotenv;
use sqlx::any::install_default_drivers;
use autometrics::prometheus_exporter;

pub mod auth;
pub mod manajemen_produk;
pub mod manajemen_pelanggan;
pub mod manajemen_pembayaran;
pub mod transaksi_penjualan;
pub mod manajemen_supplier;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/metrics")]
pub fn metrics() -> String {
    prometheus_exporter::encode_to_string().unwrap()
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let production = std::env::var("PRODUCTION").unwrap_or_else(|_| "false".to_string()) == "true";

    // CORS Configuration
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_exact(&[
            "http://127.0.0.1:3000",
            "https://a10-buildingstore-fe.koyeb.app",
            "http://localhost:3000",
            "http://192.168.1.5:3000"
        ]))
        .allow_credentials(true)
        .to_cors()
        .expect("Failed to create CORS");

    // Initialize Prometheus Exporter
    prometheus_exporter::init();

    install_default_drivers();
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = sqlx::AnyPool::connect(&database_url).await.unwrap();
    //sqlx::migrate!()
      //  .run(&db_pool)
       // .await
        //.expect("Failed to run migrations");    

    rocket::build()
        .manage(reqwest::Client::builder().build().unwrap())
        .manage(db_pool)
        .manage(production)
        .attach(cors)
        .attach(BuildingStoreDB::init())
        .attach(auth::controller::route_stage())
        .attach(manajemen_pelanggan::controller::route_stage())
        .attach(manajemen_pembayaran::controller::route_stage())
        .attach(transaksi_penjualan::controller::route_stage())
        .attach(manajemen_supplier::controller::route_stage())
        .attach(manajemen_produk::controller::route_stage())
        .mount("/", routes![index, metrics])
}