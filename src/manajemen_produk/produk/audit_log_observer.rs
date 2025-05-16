use std::fmt::Debug;
// use rocket_db_pools::sqlx::PgPool;
use super::model::Produk;
use super::events::ProdukObserver;

#[derive(Debug)]
pub struct AuditLogObserver;

impl AuditLogObserver {
    pub fn new() -> Self {
        AuditLogObserver
    }
}

impl ProdukObserver for AuditLogObserver {
    fn on_stock_changed(&self, produk: &Produk, old_stock: u32) {
        println!(
            "AUDIT LOG: Stock changed for product '{}' (ID: {:?}) from {} to {}",
            produk.nama, produk.id, old_stock, produk.stok
        );
    }
}