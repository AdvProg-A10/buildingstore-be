use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{RepositoryError};
use sqlx::{AnyPool, Row};

pub async fn ambil_semua_produk(pool: &AnyPool) -> Result<Vec<Produk>, RepositoryError> {
    let rows = sqlx::query("SELECT id, nama, kategori, CAST(harga as DOUBLE PRECISION) as harga, stok, deskripsi FROM produk ORDER BY id")
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(Produk::with_id(
            row.try_get("id")?,
            row.try_get("nama")?,
            row.try_get("kategori")?,
            row.try_get("harga")?,
            row.try_get::<i32, _>("stok")? as u32,
            row.try_get("deskripsi").map_or(None, |v: String| Some(v)),
        ));
    }
    
    Ok(products)
}

pub async fn ambil_produk_by_id(pool: &AnyPool, id: i64) -> Result<Option<Produk>, RepositoryError> {
    let row = sqlx::query("SELECT id, nama, kategori, CAST(harga as DOUBLE PRECISION) as harga, stok, deskripsi FROM produk WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(row) => Ok(Some(Produk::with_id(
            row.try_get("id")?,
            row.try_get("nama")?,
            row.try_get("kategori")?,
            row.try_get("harga")?,
            row.try_get::<i32, _>("stok")? as u32,
            row.try_get("deskripsi").map_or(None, |v: String| Some(v)),
        ))),
        None => Ok(None),
    }
}