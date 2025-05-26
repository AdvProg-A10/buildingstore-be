use rocket::serde::json::Json;
use rocket::{put, routes, Route, State};
use crate::manajemen_produk::model::{ProdukBuilder};
use crate::manajemen_produk::repository;
use super::dto::{ProdukRequest, ProdukResponse, ApiResponse};
use autometrics::autometrics;
use sqlx::AnyPool;

#[autometrics]
#[put("/produk/<id>", format = "json", data = "<request>")]
pub async fn update_produk(
    db: &State<AnyPool>,
    id: i64,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Check if product exists
    match repository::read::ambil_produk_by_id(db.inner(), id).await {
        Ok(Some(_)) => {
            // Using builder to create updated product
            let updated_produk = ProdukBuilder::new(request.nama.clone(), request.kategori.clone())
                .id(id)
                .harga(request.harga)
                .stok(request.stok.try_into().unwrap_or(0))
                .deskripsi(request.deskripsi.clone().unwrap_or_default())
                .build();
                
            match updated_produk {
                Ok(updated_produk) => {
                    match repository::update::update_produk(db.inner(), id, &updated_produk).await {
                        Ok(true) => {
                            Json(ApiResponse {
                                success: true,
                                message: Some("Berhasil memperbarui produk".to_string()),
                                data: Some(ProdukResponse::from(updated_produk)),
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
                                message: Some(format!("Gagal memperbarui produk: {}", e)),
                                data: None,
                            })
                        }
                    }
                },
                Err(e) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some(format!("Validasi gagal: {:?}", e)),
                        data: None,
                    })
                }
            }
        },
        Ok(None) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                data: None,
            })
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal mengambil produk untuk update: {}", e)),
                data: None,
            })
        }
    }
}

#[autometrics]
#[put("/produk/<id>/stok", format = "json", data = "<stok_baru>")]
pub async fn update_stok_produk(
    db: &State<AnyPool>,
    id: i64,
    stok_baru: Json<u32>
) -> Json<ApiResponse<ProdukResponse>> {
    match repository::update::update_stok(db.inner(), id, *stok_baru).await {
        Ok(true) => {
            // Get updated product to return in response
            match repository::read::ambil_produk_by_id(db.inner(), id).await {
                Ok(Some(updated_produk)) => {
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil memperbarui stok produk".to_string()),
                        data: Some(ProdukResponse::from(updated_produk)),
                    })
                },
                _ => {
                    Json(ApiResponse {
                        success: true,
                        message: Some("Stok berhasil diperbarui tetapi gagal mengambil data".to_string()),
                        data: None,
                    })
                }
            }
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
                message: Some(format!("Gagal memperbarui stok: {}", e)),
                data: None,
            })
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![update_produk, update_stok_produk]
}