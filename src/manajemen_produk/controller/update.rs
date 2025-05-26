use rocket::serde::json::Json;
use rocket::{put, routes, Route};
use crate::manajemen_produk::model::{ProdukBuilder};
use crate::manajemen_produk::repository;
use super::dto::{ProdukRequest, ProdukResponse, ApiResponse};
use autometrics::autometrics;

#[autometrics]
#[put("/produk/<id>", format = "json", data = "<request>")]
pub async fn update_produk(
    id: i64,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Check if product exists
    match repository::read::ambil_produk_by_id(id).await {
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
                    match repository::update::update_produk(id, &updated_produk).await {
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
    id: i64,
    stok_baru: Json<u32>
) -> Json<ApiResponse<ProdukResponse>> {
    match repository::update::update_stok(id, *stok_baru).await {
        Ok(true) => {
            // Get updated product to return in response
            match repository::read::ambil_produk_by_id(id).await {
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

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::asynchronous::Client;
    use rocket::serde::json::json;
    use crate::manajemen_produk::controller::{ApiResponse, ProdukResponse};
    use crate::manajemen_produk::model::Produk;
    use crate::manajemen_produk::repository;

    async fn setup_test_client() -> Client {
        let rocket = rocket::build().mount("/api", crate::manajemen_produk::controller::routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn seed_test_data() -> i64 {
        // Clean first to ensure isolation
        clean_test_data().await;
        
        let produk1 = Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        );
        
        repository::create::tambah_produk(&produk1).await.unwrap()
    }

    async fn clean_test_data() {
        let _ = repository::delete::clear_all().await;
        // Add small delay to ensure cleanup is complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_update_produk() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        let produk_id = seed_test_data().await;
        
        // Test update with valid data
        let response = client.put(format!("/api/produk/{}", produk_id))
            .header(ContentType::JSON)
            .body(json!({
                "nama": "Updated Laptop Gaming",
                "kategori": "Elektronik",
                "harga": 16_000_000.0,
                "stok": 8,
                "deskripsi": "Updated description"
            }).to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        println!("Update response: {}", body); // Debug output
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        if !json.success {
            panic!("Update failed with message: {:?}", json.message);
        }
        
        assert!(json.success, "Expected success but got failure: {:?}", json.message);
        let produk = json.data.unwrap();
        assert_eq!(produk.nama, "Updated Laptop Gaming");
        assert_eq!(produk.harga, 16_000_000.0);
        assert_eq!(produk.stok, 8);
        assert_eq!(produk.deskripsi, Some("Updated description".to_string()));
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_update_produk_not_found() {
        let client = setup_test_client().await;
        
        // Ensure clean state
        clean_test_data().await;
        
        // Test update with non-existent ID
        let response = client.put("/api/produk/9999")
            .header(ContentType::JSON)
            .body(json!({
                "nama": "Non-existent Product",
                "kategori": "Elektronik",
                "harga": 1000.0,
                "stok": 10,
                "deskripsi": "This should fail"
            }).to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(!json.success);
        assert!(json.data.is_none());
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_update_stok_produk() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        let produk_id = seed_test_data().await;
        
        // Test stock update
        let response = client.put(format!("/api/produk/{}/stok", produk_id))
            .header(ContentType::JSON)
            .body("25")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let produk = json.data.unwrap();
        assert_eq!(produk.stok, 25);
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_update_stok_produk_not_found() {
        let client = setup_test_client().await;
        
        // Ensure clean state
        clean_test_data().await;
        
        // Test stock update with non-existent ID
        let response = client.put("/api/produk/9999/stok")
            .header(ContentType::JSON)
            .body("25")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(!json.success);
        assert!(json.data.is_none());
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_update_produk_invalid_data() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        let produk_id = seed_test_data().await;
        
        // Test update with invalid data (empty name)
        let response = client.put(format!("/api/produk/{}", produk_id))
            .header(ContentType::JSON)
            .body(json!({
                "nama": "",
                "kategori": "Elektronik",
                "harga": 1000.0,
                "stok": 10,
                "deskripsi": "Invalid product"
            }).to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(!json.success, "Expected failure for invalid data but got success");
        assert!(json.data.is_none());
        
        // Clean up
        clean_test_data().await;
    }
}