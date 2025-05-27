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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}, AnyPool, Row};
    use serde_json::json;
    use crate::manajemen_produk::controller::dto::{ProdukRequest, ApiResponse, ProdukResponse};

    async fn setup_test_db() -> AnyPool {
        install_default_drivers();
        let db_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test DB");

        // Create table for testing
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS produk (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nama TEXT NOT NULL,
                kategori TEXT NOT NULL,
                harga REAL NOT NULL,
                stok INTEGER NOT NULL,
                deskripsi TEXT
            )
            "#
        )
        .execute(&db_pool)
        .await
        .expect("Failed to create test table");

        db_pool
    }

    async fn insert_test_produk(pool: &AnyPool) -> i64 {
        let result = sqlx::query(
            r#"
            INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#
        )
        .bind("Test Product")
        .bind("Test Category")
        .bind(100000.0)
        .bind(50i32)
        .bind("Test Description")
        .fetch_one(pool)
        .await
        .expect("Failed to insert test product");
        
        result.get("id")
    }

    async fn setup_rocket_client() -> (Client, AnyPool) {
        let db_pool = setup_test_db().await;
        
        let rocket = rocket::build()
            .manage(db_pool.clone())
            .mount("/api", routes![update_produk, update_stok_produk]);
            
        let client = Client::tracked(rocket)
            .await
            .expect("Valid rocket instance");
            
        (client, db_pool)
    }

   #[tokio::test]
    async fn test_update_produk_valid_request() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let request_body = json!({
            "nama": "Updated Laptop Gaming",
            "kategori": "Elektronik",
            "harga": 15000000.50,
            "stok": 25,
            "deskripsi": "Updated laptop gaming with RTX 4080"
        });

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.nama, "Updated Laptop Gaming");
        assert_eq!(produk_data.kategori, "Elektronik");
        assert_eq!(produk_data.harga, 15000000.50);
        assert_eq!(produk_data.stok, 25);
        assert_eq!(produk_data.deskripsi, Some("Updated laptop gaming with RTX 4080".to_string()));

        // Verify in database
        let row = sqlx::query("SELECT * FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch updated product");

        let nama: String = row.get("nama");
        let kategori: String = row.get("kategori");
        let harga: f64 = row.get("harga");
        let stok: i32 = row.get("stok");
        
        assert_eq!(nama, "Updated Laptop Gaming");
        assert_eq!(kategori, "Elektronik");
        assert!((harga - 15000000.50).abs() < f64::EPSILON);
        assert_eq!(stok, 25);
    }

    #[tokio::test]
    async fn test_update_produk_not_found() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Non-existent Product",
            "kategori": "Test",
            "harga": 100000.0,
            "stok": 10,
            "deskripsi": "This should fail"
        });

        let response = client
            .put("/api/produk/999")
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.data.is_none());
        assert!(response_body.message.unwrap().contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn test_update_produk_negative_stock_becomes_zero() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let request_body = json!({
            "nama": "Updated Product",
            "kategori": "Test",
            "harga": 75000.0,
            "stok": -5, // Negative stock should become 0
            "deskripsi": "Test with negative stock"
        });

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.stok, 0); // Should be converted to 0
    }

    #[tokio::test]
    async fn test_update_produk_empty_name_should_fail() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let request_body = json!({
            "nama": "", // Empty name should fail validation
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 1,
            "deskripsi": "Test product"
        });

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        // Should fail due to validation
        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.message.unwrap().contains("Validasi gagal"));
    }

    #[tokio::test]
    async fn test_update_produk_negative_price_should_fail() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let request_body = json!({
            "nama": "Updated Product",
            "kategori": "Test",
            "harga": -100.0, // Negative price should fail
            "stok": 1,
            "deskripsi": "This should fail"
        });

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        // Should fail due to validation
        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.message.unwrap().contains("Validasi gagal"));
    }

    #[tokio::test]
    async fn test_update_produk_special_characters() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let request_body = json!({
            "nama": "Café Latte & Cappuccino™ Updated",
            "kategori": "Minuman & Makanan",
            "harga": 55000.75,
            "stok": 75,
            "deskripsi": "Updated premium coffee blend with açaí & ginseng"
        });

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.nama, "Café Latte & Cappuccino™ Updated");
        assert_eq!(produk_data.kategori, "Minuman & Makanan");
    }

    #[tokio::test]
    async fn test_update_produk_large_values() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let request_body = json!({
            "nama": "Enterprise Server Updated",
            "kategori": "Server Hardware",
            "harga": 1999999999.99,
            "stok": 999999,
            "deskripsi": "Updated enterprise server with enhanced features"
        });

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.stok, 999999);
        assert!((produk_data.harga - 1999999999.99).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_update_produk_invalid_json() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let invalid_json = "{ invalid json structure";

        let path = format!("/api/produk/{}", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(invalid_json)
            .dispatch()
            .await;

        // Should return bad request for invalid JSON
        assert_eq!(response.status(), rocket::http::Status::BadRequest);
    }

    // Tests for update_stok_produk endpoint

    #[tokio::test]
    async fn test_update_stok_valid_request() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let new_stok = 150u32;

        let path = format!("/api/produk/{}/stok", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(new_stok.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.stok, 150);

        // Verify in database
        let row = sqlx::query("SELECT stok FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch updated product");

        let stok: i32 = row.get("stok");
        assert_eq!(stok, 150);
    }

    #[tokio::test]
    async fn test_update_stok_zero_value() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let new_stok = 0u32;

        let path = format!("/api/produk/{}/stok", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(new_stok.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.stok, 0);
    }

    #[tokio::test]
    async fn test_update_stok_large_value() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let new_stok = 999999u32;

        let path = format!("/api/produk/{}/stok", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(new_stok.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());
        
        let produk_data = response_body.data.unwrap();
        assert_eq!(produk_data.stok, 999999);
    }

    #[tokio::test]
    async fn test_update_stok_invalid_json() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let invalid_json = "invalid_number";

        let path = format!("/api/produk/{}/stok", product_id);
        let response = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(invalid_json)
            .dispatch()
            .await;

        // Should return bad request for invalid JSON
        assert_eq!(response.status(), rocket::http::Status::BadRequest);
    }

    #[tokio::test]
    async fn test_multiple_updates_on_same_product() {
        let (client, db_pool) = setup_rocket_client().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        // First update - full product update
        let request_body1 = json!({
            "nama": "First Update",
            "kategori": "Category 1",
            "harga": 100000.0,
            "stok": 10,
            "deskripsi": "First update description"
        });

        let path = format!("/api/produk/{}", product_id);
        let response1 = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body1.to_string())
            .dispatch()
            .await;

        let status1 = response1.status();
        let response_body1: ApiResponse<ProdukResponse> = response1.into_json().await.unwrap();
        assert_eq!(status1, rocket::http::Status::Ok);
        assert!(response_body1.success);

        // Second update - stock only
        let new_stok = 25u32;
        let path = format!("/api/produk/{}/stok", product_id);
        let response2 = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(new_stok.to_string())
            .dispatch()
            .await;

        assert_eq!(response2.status(), rocket::http::Status::Ok);
        let response_body2: ApiResponse<ProdukResponse> = response2.into_json().await.unwrap();
        assert!(response_body2.success);
        assert_eq!(response_body2.data.unwrap().stok, 25);

        // Third update - full product update again
        let request_body3 = json!({
            "nama": "Final Update",
            "kategori": "Final Category",
            "harga": 200000.0,
            "stok": 30,
            "deskripsi": "Final update description"
        });

        let path = format!("/api/produk/{}", product_id);
        let response3 = client
            .put(&path)
            .header(rocket::http::ContentType::JSON)
            .body(request_body3.to_string())
            .dispatch()
            .await;

        assert_eq!(response3.status(), rocket::http::Status::Ok);
        let response_body3: ApiResponse<ProdukResponse> = response3.into_json().await.unwrap();
        assert!(response_body3.success);
        
        let final_data = response_body3.data.unwrap();
        assert_eq!(final_data.nama, "Final Update");
        assert_eq!(final_data.kategori, "Final Category");
        assert_eq!(final_data.harga, 200000.0);
        assert_eq!(final_data.stok, 30);

        // Verify final state in database
        let row = sqlx::query("SELECT * FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch updated product");

        let nama: String = row.get("nama");
        let kategori: String = row.get("kategori");
        let harga: f64 = row.get("harga");
        let stok: i32 = row.get("stok");
        
        assert_eq!(nama, "Final Update");
        assert_eq!(kategori, "Final Category");
        assert_eq!(harga, 200000.0);
        assert_eq!(stok, 30);
    }
}