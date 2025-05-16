pub mod auth;
pub mod manajemen_produk;
pub mod transaksi_penjualan;
pub mod manajemen_supplier;
pub mod manajemen_pelanggan;
pub mod manajemen_pembayaran;

use rocket_db_pools::Database;

#[derive(Database)]
#[database("buildingstore")]
pub struct BuildingStoreDB(sqlx::PgPool);