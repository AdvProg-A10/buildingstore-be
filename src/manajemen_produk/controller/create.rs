use rocket::serde::json::Json;
use rocket::{post, routes, Route, State};
use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository;
use super::dto::{ProdukRequest, ProdukResponse, ApiResponse};
use autometrics::autometrics;
use sqlx::AnyPool;

#[autometrics]
#[post("/produk", format = "json", data = "<request>")]
pub async fn tambah_produk(
    db: &State<AnyPool>,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Validasi stok tidak boleh negatif
    let stok = if request.stok < 0 { 0 } else { request.stok as u32 };
    
    let produk = Produk::new(
        request.nama.clone(),
        request.kategori.clone(),
        request.harga,
        stok,
        request.deskripsi.clone(),
    );

    match repository::create::tambah_produk(db.inner(), &produk).await {
        Ok(id) => {
            // Ambil produk yang baru dibuat untuk response
            match repository::read::ambil_produk_by_id(db.inner(), id).await {
                Ok(Some(created_produk)) => {
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil menambahkan produk".to_string()),
                        data: Some(ProdukResponse::from(created_produk)),
                    })
                },
                Ok(None) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some("Produk berhasil dibuat tetapi tidak ditemukan".to_string()),
                        data: None,
                    })
                },
                Err(e) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some(format!("Produk berhasil dibuat tetapi gagal mengambil data: {}", e)),
                        data: None,
                    })
                }
            }
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal menambahkan produk: {}", e)),
                data: None,
            })
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![tambah_produk]
}