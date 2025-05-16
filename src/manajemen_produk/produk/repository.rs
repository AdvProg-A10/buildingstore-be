// use rocket_db_pools::Connection;
use rocket_db_pools::sqlx::{self, PgConnection, Error, Row};
// use crate::BuildingStoreDB;
use super::model::Produk;

pub struct ProdukRepository;

impl ProdukRepository {
    pub async fn tambah_produk(db: &mut PgConnection, produk: &Produk) -> Result<i64, Error> {
        let result = sqlx::query(
            "INSERT INTO produk (nama, kategori, harga, stok, deskripsi) VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(&produk.nama)
        .bind(&produk.kategori)
        .bind(produk.harga)
        .bind(produk.stok as i32)  // Cast to i32
        .bind(&produk.deskripsi)
        .fetch_one(db)
        .await?;
    
        Ok(result.get("id"))
    }
    
    pub async fn ambil_semua_produk(db: &mut PgConnection) -> Result<Vec<Produk>, Error> {
        let rows = sqlx::query(
            "SELECT id, nama, kategori, harga, stok, deskripsi FROM produk"
        )
        .fetch_all(db)
        .await?;
        
        let produk_list = rows.iter().map(|row| {
            Produk::with_id(
                row.get("id"),
                row.get("nama"),
                row.get("kategori"),
                row.get("harga"),
                row.get::<i32, _>("stok") as u32,  // Convert i32 to u32
                row.get("deskripsi"),
            )
        }).collect();
        
        Ok(produk_list)
    }
    
    pub async fn ambil_produk_by_id(db: &mut PgConnection, id: i64) -> Result<Option<Produk>, Error> {
        let maybe_row = sqlx::query(
            "SELECT id, nama, kategori, harga, stok, deskripsi FROM produk WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(db)
        .await?;
        
        match maybe_row {
            Some(row) => Ok(Some(Produk::with_id(
                row.get("id"),
                row.get("nama"),
                row.get("kategori"),
                row.get("harga"),
                row.get::<i32, _>("stok") as u32,  // Convert i32 to u32
                row.get("deskripsi"),
            ))),
            None => Ok(None)
        }
    }
    
    pub async fn filter_produk_by_kategori(db: &mut PgConnection, kategori: &str) -> Result<Vec<Produk>, Error> {
        let rows = sqlx::query(
            "SELECT id, nama, kategori, harga, stok, deskripsi FROM produk WHERE kategori = $1"
        )
        .bind(kategori)
        .fetch_all(db)
        .await?;
        
        let produk_list = rows.iter().map(|row| {
            Produk::with_id(
                row.get("id"),
                row.get("nama"),
                row.get("kategori"),
                row.get("harga"),
                row.get::<i32, _>("stok") as u32,  // Convert i32 to u32
                row.get("deskripsi"),
            )
        }).collect();
        
        Ok(produk_list)
    }
    
    pub async fn update_produk(db: &mut PgConnection, id: i64, produk: &Produk) -> Result<bool, Error> {
        let result = sqlx::query(
            "UPDATE produk SET nama = $1, kategori = $2, harga = $3, stok = $4, deskripsi = $5 WHERE id = $6"
        )
        .bind(&produk.nama)
        .bind(&produk.kategori)
        .bind(produk.harga)
        .bind(produk.stok as i32)  // Cast to i32
        .bind(&produk.deskripsi)
        .bind(id)
        .execute(db)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn hapus_produk(db: &mut PgConnection, id: i64) -> Result<bool, Error> {
        let result = sqlx::query("DELETE FROM produk WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
            
        Ok(result.rows_affected() > 0)
    }
    
    // Tambahan: Metode untuk mencari produk berdasarkan harga
    pub async fn filter_produk_by_price_range(
        db: &mut PgConnection, 
        min_price: f64, 
        max_price: f64
    ) -> Result<Vec<Produk>, Error> {
        let rows = sqlx::query(
            "SELECT id, nama, kategori, harga, stok, deskripsi FROM produk WHERE harga >= $1 AND harga <= $2"
        )
        .bind(min_price)
        .bind(max_price)
        .fetch_all(db)
        .await?;
        
        let produk_list = rows.iter().map(|row| {
            Produk::with_id(
                row.get("id"),
                row.get("nama"),
                row.get("kategori"),
                row.get("harga"),
                row.get::<i32, _>("stok") as u32,  // Convert i32 to u32
                row.get("deskripsi"),
            )
        }).collect();
        
        Ok(produk_list)
    }
    
    // Tambahan: Metode untuk mencari produk berdasarkan stok
    pub async fn filter_produk_by_stock_availability(db: &mut PgConnection, min_stock: u32) -> Result<Vec<Produk>, Error> {
        let rows = sqlx::query(
            "SELECT id, nama, kategori, harga, stok, deskripsi FROM produk WHERE stok >= $1"
        )
        .bind(min_stock as i32)  // Cast to i32
        .fetch_all(db)
        .await?;
        
        let produk_list = rows.iter().map(|row| {
            Produk::with_id(
                row.get("id"),
                row.get("nama"),
                row.get("kategori"),
                row.get("harga"),
                row.get::<i32, _>("stok") as u32,  // Convert i32 to u32
                row.get("deskripsi"),
            )
        }).collect();
        
        Ok(produk_list)
    }
}