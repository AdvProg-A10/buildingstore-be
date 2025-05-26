use rocket::serde::json::Json;
use rocket::{get, routes, Route, State};
use crate::manajemen_produk::repository;
use super::dto::{ProdukResponse, ApiResponse};
use autometrics::autometrics;
use sqlx::AnyPool;

#[autometrics]
#[get("/produk")]
pub async fn list_produk(db: &State<AnyPool>) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match repository::read::ambil_semua_produk(db.inner()).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(ProdukResponse::from)
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some("Berhasil mengambil daftar produk".to_string()),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil daftar produk: {}", e)),
            data: None,
        }),
    }
}

#[autometrics]
#[get("/produk/<id>")]
pub async fn detail_produk(db: &State<AnyPool>, id: i64) -> Json<ApiResponse<ProdukResponse>> {
    match repository::read::ambil_produk_by_id(db.inner(), id).await {
        Ok(Some(produk)) => Json(ApiResponse {
            success: true,
            message: Some("Berhasil mengambil detail produk".to_string()),
            data: Some(ProdukResponse::from(produk)),
        }),
        Ok(None) => Json(ApiResponse {
            success: false,
            message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil detail produk: {}", e)),
            data: None,
        }),
    }
}

pub fn routes() -> Vec<Route> {
    routes![list_produk, detail_produk]
}