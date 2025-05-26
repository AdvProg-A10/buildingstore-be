use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{validate_produk, RepositoryError};
use sqlx::AnyPool;

pub async fn update_produk(pool: &AnyPool, id: i64, produk: &Produk) -> Result<bool, RepositoryError> {
    // Validasi input
    validate_produk(produk)?;
    
    let result = sqlx::query(
        r#"
        UPDATE produk 
        SET nama = $1, kategori = $2, harga = $3, stok = $4, deskripsi = $5
        WHERE id = $6
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i32)
    .bind(&produk.deskripsi)
    .bind(id)
    .execute(pool)
    .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_stok(pool: &AnyPool, id: i64, new_stok: u32) -> Result<bool, RepositoryError> {
    let result = sqlx::query("UPDATE produk SET stok = $1 WHERE id = $2")
        .bind(new_stok as i32)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_harga(pool: &AnyPool, id: i64, new_harga: f64) -> Result<bool, RepositoryError> {
    if new_harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    let result = sqlx::query("UPDATE produk SET harga = $1 WHERE id = $2")
        .bind(new_harga)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}