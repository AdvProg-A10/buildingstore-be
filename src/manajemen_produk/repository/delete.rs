use crate::manajemen_produk::repository::dto::RepositoryError;
use sqlx::AnyPool;

pub async fn hapus_produk(pool: &AnyPool, id: i64) -> Result<bool, RepositoryError> {
    let result = sqlx::query("DELETE FROM produk WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}

pub async fn clear_all(pool: &AnyPool) -> Result<(), RepositoryError> {
    // Start transaction
    let mut tx = pool.begin().await?;
    
    // Clear all products
    sqlx::query("DELETE FROM produk")
        .execute(&mut *tx)
        .await?;
    
    // Reset sequence counter (PostgreSQL way)
    sqlx::query("ALTER SEQUENCE produk_id_seq RESTART WITH 1")
        .execute(&mut *tx)
        .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(())
}