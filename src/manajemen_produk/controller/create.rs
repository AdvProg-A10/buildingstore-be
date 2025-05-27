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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}, AnyPool};
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

    async fn setup_rocket_client() -> (Client, AnyPool) {
        let db_pool = setup_test_db().await;
        
        let rocket = rocket::build()
            .manage(db_pool.clone())
            .mount("/api", routes![tambah_produk]);
            
        let client = Client::tracked(rocket)
            .await
            .expect("Valid rocket instance");
            
        (client, db_pool)
    }

    #[tokio::test]
    async fn test_tambah_produk_valid_request() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Laptop Gaming ASUS",
            "kategori": "Elektronik",
            "harga": 15000000.50,
            "stok": 10,
            "deskripsi": "Laptop gaming high-end dengan RTX 4080"
        });

        let response = client
            .post("/api/produk")
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
        assert_eq!(produk_data.nama, "Laptop Gaming ASUS");
        assert_eq!(produk_data.kategori, "Elektronik");
        assert_eq!(produk_data.harga, 15000000.50);
        assert_eq!(produk_data.stok, 10);
        assert_eq!(produk_data.deskripsi, Some("Laptop gaming high-end dengan RTX 4080".to_string()));
    }

    #[tokio::test]
    async fn test_tambah_produk_minimal_data() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Mouse Wireless",
            "kategori": "Aksesoris",
            "harga": 150000.0,
            "stok": 50,
            "deskripsi": null
        });

        let response = client
            .post("/api/produk")
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
        assert_eq!(produk_data.nama, "Mouse Wireless");
        assert_eq!(produk_data.kategori, "Aksesoris");
        assert_eq!(produk_data.harga, 150000.0);
        assert_eq!(produk_data.stok, 50);
        assert_eq!(produk_data.deskripsi, None);
    }

    #[tokio::test]
    async fn test_tambah_produk_negative_stock_becomes_zero() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Keyboard Mechanical",
            "kategori": "Aksesoris",
            "harga": 750000.99,
            "stok": -5, // Negative stock should become 0
            "deskripsi": "Keyboard mechanical blue switch"
        });

        let response = client
            .post("/api/produk")
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
    async fn test_tambah_produk_special_characters() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Café Latte & Cappuccino™",
            "kategori": "Minuman & Makanan",
            "harga": 45000.50,
            "stok": 100,
            "deskripsi": "Premium coffee blend with special ingredients: açaí, ginseng & organic milk"
        });

        let response = client
            .post("/api/produk")
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
        assert_eq!(produk_data.nama, "Café Latte & Cappuccino™");
        assert_eq!(produk_data.kategori, "Minuman & Makanan");
    }

    #[tokio::test]
    async fn test_tambah_produk_invalid_json() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let invalid_json = "{ invalid json structure";

        let response = client
            .post("/api/produk")
            .header(rocket::http::ContentType::JSON)
            .body(invalid_json)
            .dispatch()
            .await;

        // Should return bad request for invalid JSON
        assert_eq!(response.status(), rocket::http::Status::BadRequest);
    }

    #[tokio::test]
    async fn test_tambah_produk_large_values() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Server Enterprise",
            "kategori": "Server",
            "harga": 999999999.99,
            "stok": 999999,
            "deskripsi": "High-end enterprise server with redundant systems and 24/7 support warranty"
        });

        let response = client
            .post("/api/produk")
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
        assert!((produk_data.harga - 999999999.99).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_tambah_produk_zero_stock() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Pre-order Item",
            "kategori": "Games",
            "harga": 500000.0,
            "stok": 0, // Zero stock
            "deskripsi": "Pre-order game, will be available soon"
        });

        let response = client
            .post("/api/produk")
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
        assert_eq!(produk_data.stok, 0);
    }

    #[tokio::test]
    async fn test_tambah_produk_empty_strings_should_fail() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "", // Empty name should fail validation
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 1,
            "deskripsi": "Test product"
        });

        let response = client
            .post("/api/produk")
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
    }

    #[tokio::test]
    async fn test_tambah_produk_negative_price_should_fail() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        let request_body = json!({
            "nama": "Free Item",
            "kategori": "Test",
            "harga": -100.0, // Negative price should fail
            "stok": 1,
            "deskripsi": "This should fail"
        });

        let response = client
            .post("/api/produk")
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
    }

    #[tokio::test]
    async fn test_tambah_multiple_products_sequentially() {
        let (client, db_pool) = setup_rocket_client().await;
        
        let products = vec![
            json!({
                "nama": "Product 1",
                "kategori": "Category A",
                "harga": 100000.0,
                "stok": 10,
                "deskripsi": "First product"
            }),
            json!({
                "nama": "Product 2", 
                "kategori": "Category B",
                "harga": 200000.0,
                "stok": 20,
                "deskripsi": "Second product"
            }),
            json!({
                "nama": "Product 3",
                "kategori": "Category C", 
                "harga": 300000.0,
                "stok": 30,
                "deskripsi": "Third product"
            })
        ];

        let mut created_ids = Vec::new();

        // Create all products
        for product in products {
            let response = client
                .post("/api/produk")
                .header(rocket::http::ContentType::JSON)
                .body(product.to_string())
                .dispatch()
                .await;

            assert_eq!(response.status(), rocket::http::Status::Ok);
            
            let response_body: ApiResponse<ProdukResponse> = response
                .into_json()
                .await
                .expect("Valid JSON response");

            assert!(response_body.success);
            let created_product = response_body.data.unwrap();
            created_ids.push(created_product.id.unwrap());
        }

        // Verify all products were created with different IDs
        assert_eq!(created_ids.len(), 3);
        assert_ne!(created_ids[0], created_ids[1]);
        assert_ne!(created_ids[1], created_ids[2]);
        assert_ne!(created_ids[0], created_ids[2]);

        // Verify all products exist in database
        let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products");
            
        assert_eq!(total_count, 3);
    }
}