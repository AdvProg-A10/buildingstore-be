use rocket::serde::json::Json;
use rocket::{delete, routes, Route, State};
use crate::manajemen_produk::repository;
use super::dto::ApiResponse;
use autometrics::autometrics;
use sqlx::AnyPool;

#[autometrics]
#[delete("/produk/<id>")]
pub async fn hapus_produk(
    db: &State<AnyPool>,
    id: i64
) -> Json<ApiResponse<()>> {
    match repository::delete::hapus_produk(db.inner(), id).await {
        Ok(true) => {
            Json(ApiResponse {
                success: true,
                message: Some(format!("Produk dengan ID {} berhasil dihapus", id)),
                data: None,
            })
        },
        Ok(false) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                data: None,
            })
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal menghapus produk: {}", e)),
                data: None,
            })
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![hapus_produk]
}