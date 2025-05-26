use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{validate_produk, RepositoryError};
use sqlx::{AnyPool, Row};

pub async fn tambah_produk(pool: &AnyPool, produk: &Produk) -> Result<i64, RepositoryError> {
    // Validasi terlebih dahulu
    validate_produk(produk)?;
    
    let result = sqlx::query(
        r#"
        INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i32)
    .bind(&produk.deskripsi)
    .fetch_one(pool)
    .await?;
    
    Ok(result.get("id"))
}