use sqlx::{PgPool, Row};
use std::sync::OnceLock;
use std::error::Error as StdError;
use std::fmt;
use crate::manajemen_produk::model::Produk;

// Global database connection pool
static DB_POOL: OnceLock<PgPool> = OnceLock::new();

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

// Database initialization
pub async fn init_database() -> Result<(), RepositoryError> {
    // Use PostgreSQL connection string - adjust as needed for your setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/test_db".to_string());
    
    let pool = PgPool::connect(&database_url).await?;
    
    // Create products table with PostgreSQL syntax
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS produk (
            id BIGSERIAL PRIMARY KEY,
            nama VARCHAR NOT NULL,
            kategori VARCHAR NOT NULL,
            harga DECIMAL(15,2) NOT NULL,
            stok INTEGER NOT NULL,
            deskripsi TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Create trigger for updated_at (PostgreSQL syntax)
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = CURRENT_TIMESTAMP;
            RETURN NEW;
        END;
        $$ language 'plpgsql';
        
        DROP TRIGGER IF EXISTS update_produk_updated_at ON produk;
        CREATE TRIGGER update_produk_updated_at
            BEFORE UPDATE ON produk
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
        "#
    )
    .execute(&pool)
    .await?;

    DB_POOL.set(pool).map_err(|_| RepositoryError::Other("Failed to set database pool".to_string()))?;
    Ok(())
}

// Get database pool
pub fn get_db_pool() -> Result<&'static PgPool, RepositoryError> {
    DB_POOL.get().ok_or(RepositoryError::Other("Database not initialized".to_string()))
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
    
    Ok(())
}

// Convert database row to Produk
pub fn row_to_produk(row: &sqlx::postgres::PgRow) -> Result<Produk, sqlx::Error> {
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
pub async fn get_store_stats() -> Result<(i64, i64), RepositoryError> {
    let pool = get_db_pool()?;
    
    let row = sqlx::query("SELECT COUNT(*) as count, COALESCE(MAX(id), 0) as max_id FROM produk")
        .fetch_one(pool)
        .await?;
    
    let count: i64 = row.try_get("count")?;
    let max_id: i64 = row.try_get("max_id")?;
    
    Ok((count, max_id))
}