use sqlx::{AnyPool, Row};
use std::error::Error as StdError;
use std::fmt;
use crate::manajemen_produk::model::Produk;
use rocket::State;

// Error types
#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    DatabaseError(sqlx::Error),
    ValidationError(String),
    Other(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Record not found"),
            RepositoryError::DatabaseError(e) => write!(f, "Database error: {}", e),
            RepositoryError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RepositoryError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for RepositoryError {}

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        RepositoryError::DatabaseError(error)
    }
}

// Helper function untuk mendapatkan pool dari Rocket State
pub fn get_db_pool_from_state(db: &State<AnyPool>) -> &AnyPool {
    db.inner()
}

// Validation helpers
pub fn validate_produk(produk: &Produk) -> Result<(), RepositoryError> {
    if produk.nama.trim().is_empty() {
        return Err(RepositoryError::ValidationError("Nama produk tidak boleh kosong".to_string()));
    }
    
    if produk.kategori.trim().is_empty() {
        return Err(RepositoryError::ValidationError("Kategori tidak boleh kosong".to_string()));
    }
    
    if produk.harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    if produk.stok < 0 {
        return Err(RepositoryError::ValidationError("Stok tidak boleh negatif".to_string()));
    }
    
    // Validasi panjang string sesuai database constraints
    if produk.nama.len() > 255 {
        return Err(RepositoryError::ValidationError("Nama produk terlalu panjang (maksimal 255 karakter)".to_string()));
    }
    
    if produk.kategori.len() > 100 {
        return Err(RepositoryError::ValidationError("Kategori terlalu panjang (maksimal 100 karakter)".to_string()));
    }
    
    Ok(())
}

// Convert database row to Produk - support untuk AnyRow
pub fn row_to_produk(row: &sqlx::any::AnyRow) -> Result<Produk, sqlx::Error> {
    Ok(Produk::with_id(
        row.try_get("id")?,
        row.try_get("nama")?,
        row.try_get("kategori")?,
        row.try_get("harga")?,
        row.try_get::<i32, _>("stok")? as u32,
        row.try_get("deskripsi")?,
    ))
}

// Statistics helper
pub async fn get_store_stats(pool: &AnyPool) -> Result<(i64, i64), RepositoryError> {
    let row = sqlx::query("SELECT COUNT(*) as count, COALESCE(MAX(id), 0) as max_id FROM produk")
        .fetch_one(pool)
        .await?;
    
    let count: i64 = row.try_get("count")?;
    let max_id: i64 = row.try_get("max_id")?;
    
    Ok((count, max_id))
}