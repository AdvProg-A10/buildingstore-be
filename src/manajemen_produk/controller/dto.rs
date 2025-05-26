use rocket::serde::{Deserialize, Serialize};
use crate::manajemen_produk::model::Produk;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProdukRequest {
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: i32,
    pub deskripsi: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProdukResponse {
    pub id: Option<i64>,
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

impl From<Produk> for ProdukResponse {
    fn from(produk: Produk) -> Self {
        Self {
            id: produk.id,
            nama: produk.nama,
            kategori: produk.kategori,
            harga: produk.harga,
            stok: produk.stok,
            deskripsi: produk.deskripsi,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}