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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}, AnyPool};
    use crate::manajemen_produk::controller::dto::{ApiResponse, ProdukResponse};

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

    async fn insert_test_data(pool: &AnyPool) {
        // Insert sample data for testing
        let test_products = vec![
            ("Laptop Gaming", "Elektronik", 15000000.50, 10, Some("Laptop gaming high-end dengan RTX 4080")),
            ("Mouse Wireless", "Aksesoris", 150000.00, 50, None),
            ("Keyboard Mechanical", "Aksesoris", 750000.99, 0, Some("Keyboard mechanical blue switch")),
            ("iPhone 15", "Smartphone", 15000000.00, 25, Some("Latest iPhone model")),
            ("Samsung Galaxy S24", "Smartphone", 12000000.00, 30, Some("Latest Samsung flagship")),
        ];

        for (nama, kategori, harga, stok, deskripsi) in test_products {
            sqlx::query(
                r#"
                INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind(nama)
            .bind(kategori)
            .bind(harga)
            .bind(stok)
            .bind(deskripsi)
            .execute(pool)
            .await
            .expect("Failed to insert test data");
        }
    }

    async fn setup_rocket_client() -> (Client, AnyPool) {
        let db_pool = setup_test_db().await;
        
        let rocket = rocket::build()
            .manage(db_pool.clone())
            .mount("/api", routes![list_produk, detail_produk]);
            
        let client = Client::tracked(rocket)
            .await
            .expect("Valid rocket instance");
            
        (client, db_pool)
    }

    #[tokio::test]
    async fn test_list_produk_with_data() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        let response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<Vec<ProdukResponse>> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert_eq!(response_body.message.as_ref().unwrap(), "Berhasil mengambil daftar produk");
        assert!(response_body.data.is_some());

        let products = response_body.data.unwrap();
        assert_eq!(products.len(), 5);

        // Verify first product
        let first_product = &products[0];
        assert_eq!(first_product.nama, "Laptop Gaming");
        assert_eq!(first_product.kategori, "Elektronik");
        assert!((first_product.harga - 15000000.50).abs() < f64::EPSILON);
        assert_eq!(first_product.stok, 10);
        assert_eq!(first_product.deskripsi, Some("Laptop gaming high-end dengan RTX 4080".to_string()));

        // Verify second product (with None description)
        let second_product = &products[1];
        assert_eq!(second_product.nama, "Mouse Wireless");
        assert_eq!(second_product.kategori, "Aksesoris");
        assert!((second_product.harga - 150000.00).abs() < f64::EPSILON);
        assert_eq!(second_product.stok, 50);
        assert_eq!(second_product.deskripsi, None);

        // Verify third product (with zero stock)
        let third_product = &products[2];
        assert_eq!(third_product.nama, "Keyboard Mechanical");
        assert_eq!(third_product.stok, 0);
    }

    #[tokio::test]
    async fn test_list_produk_empty_table() {
        let (client, _db_pool) = setup_rocket_client().await;
        // Don't insert any data

        let response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<Vec<ProdukResponse>> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());

        let products = response_body.data.unwrap();
        assert_eq!(products.len(), 0);
        assert!(products.is_empty());
    }

    #[tokio::test]
    async fn test_list_produk_order_by_id() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        let response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<Vec<ProdukResponse>> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        let products = response_body.data.unwrap();
        
        // Verify products are ordered by ID
        for i in 1..products.len() {
            if let (Some(prev_id), Some(curr_id)) = (products[i-1].id, products[i].id) {
                assert!(prev_id < curr_id, "Products should be ordered by ID");
            }
        }
    }

    #[tokio::test]
    async fn test_detail_produk_existing() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Get the first product ID from database
        let first_product_id: i64 = sqlx::query_scalar("SELECT id FROM produk ORDER BY id LIMIT 1")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to get first product ID");

        let response = client
            .get(format!("/api/produk/{}", first_product_id))
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert_eq!(response_body.message.as_ref().unwrap(), "Berhasil mengambil detail produk");
        assert!(response_body.data.is_some());

        let found_product = response_body.data.unwrap();
        assert_eq!(found_product.id, Some(first_product_id));
        assert_eq!(found_product.nama, "Laptop Gaming");
        assert_eq!(found_product.kategori, "Elektronik");
        assert!((found_product.harga - 15000000.50).abs() < f64::EPSILON);
        assert_eq!(found_product.stok, 10);
    }

    #[tokio::test]
    async fn test_detail_produk_nonexistent() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Try to get a product with ID that doesn't exist
        let response = client
            .get("/api/produk/999999")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert_eq!(response_body.message.as_ref().unwrap(), "Produk dengan ID 999999 tidak ditemukan");
        assert!(response_body.data.is_none());
    }

    #[tokio::test]
    async fn test_detail_produk_zero_stock() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Find the keyboard with zero stock
        let keyboard_id: i64 = sqlx::query_scalar("SELECT id FROM produk WHERE nama = 'Keyboard Mechanical'")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to get keyboard ID");

        let response = client
            .get(format!("/api/produk/{}", keyboard_id))
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());

        let found_product = response_body.data.unwrap();
        assert_eq!(found_product.nama, "Keyboard Mechanical");
        assert_eq!(found_product.stok, 0);
    }

    #[tokio::test]
    async fn test_detail_produk_with_null_description() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Find the mouse with null description
        let mouse_id: i64 = sqlx::query_scalar("SELECT id FROM produk WHERE nama = 'Mouse Wireless'")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to get mouse ID");

        let response = client
            .get(format!("/api/produk/{}", mouse_id))
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.data.is_some());

        let found_product = response_body.data.unwrap();
        assert_eq!(found_product.nama, "Mouse Wireless");
        assert_eq!(found_product.deskripsi, None);
    }

    #[tokio::test]
    async fn test_detail_produk_negative_id() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Try to get a product with negative ID
        let response = client
            .get("/api/produk/-1")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert_eq!(response_body.message.as_ref().unwrap(), "Produk dengan ID -1 tidak ditemukan");
        assert!(response_body.data.is_none());
    }

    #[tokio::test]
    async fn test_list_produk_with_special_characters() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert product with special characters
        sqlx::query(
            r#"
            INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind("Café Latte & Cappuccino™")
        .bind("Minuman & Makanan")
        .bind(45000.50)
        .bind(100)
        .bind("Premium coffee blend with special ingredients: açaí, ginseng & organic milk")
        .execute(&db_pool)
        .await
        .expect("Failed to insert test data with special characters");

        let response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<Vec<ProdukResponse>> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        let products = response_body.data.unwrap();
        assert_eq!(products.len(), 1);

        let product = &products[0];
        assert_eq!(product.nama, "Café Latte & Cappuccino™");
        assert_eq!(product.kategori, "Minuman & Makanan");
        assert_eq!(product.deskripsi, Some("Premium coffee blend with special ingredients: açaí, ginseng & organic milk".to_string()));
    }

    #[tokio::test]
    async fn test_list_produk_with_large_values() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert product with large values
        sqlx::query(
            r#"
            INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind("Server Enterprise")
        .bind("Server")
        .bind(999999999.99)
        .bind(999999)
        .bind("High-end enterprise server")
        .execute(&db_pool)
        .await
        .expect("Failed to insert test data with large values");

        let response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<Vec<ProdukResponse>> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        let products = response_body.data.unwrap();
        assert_eq!(products.len(), 1);

        let product = &products[0];
        assert_eq!(product.nama, "Server Enterprise");
        assert!((product.harga - 999999999.99).abs() < 1.0); // Floating point comparison
        assert_eq!(product.stok, 999999);
    }

    #[tokio::test]
    async fn test_multiple_endpoints_consistency() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Get all products via list endpoint
        let list_response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(list_response.status(), rocket::http::Status::Ok);
        
        let list_response_body: ApiResponse<Vec<ProdukResponse>> = list_response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(list_response_body.success);
        let all_products = list_response_body.data.unwrap();
        assert_eq!(all_products.len(), 5);

        // Get each product individually and compare
        for product in &all_products {
            let detail_response = client
                .get(format!("/api/produk/{}", product.id.unwrap()))
                .dispatch()
                .await;
            
            assert_eq!(detail_response.status(), rocket::http::Status::Ok);
            
            let detail_response_body: ApiResponse<ProdukResponse> = detail_response
                .into_json()
                .await
                .expect("Valid JSON response");
            
            assert!(detail_response_body.success);
            let individual_product = detail_response_body.data.unwrap();
            
            assert_eq!(individual_product.id, product.id);
            assert_eq!(individual_product.nama, product.nama);
            assert_eq!(individual_product.kategori, product.kategori);
            assert!((individual_product.harga - product.harga).abs() < f64::EPSILON);
            assert_eq!(individual_product.stok, product.stok);
            assert_eq!(individual_product.deskripsi, product.deskripsi);
        }
    }

    #[tokio::test]
    async fn test_list_produk_response_structure() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        let response = client
            .get("/api/produk")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<Vec<ProdukResponse>> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        // Verify response structure
        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.data.is_some());

        let products = response_body.data.unwrap();
        for product in products {
            // Verify each product has required fields
            assert!(product.id.is_some());
            assert!(!product.nama.is_empty());
            assert!(!product.kategori.is_empty());
            assert!(product.harga >= 0.0);
            // stok and deskripsi can be any value (including 0 and None)
        }
    }

    #[tokio::test]
    async fn test_detail_produk_response_structure() {
        let (client, db_pool) = setup_rocket_client().await;
        insert_test_data(&db_pool).await;

        // Get any existing product ID
        let product_id: i64 = sqlx::query_scalar("SELECT id FROM produk ORDER BY id LIMIT 1")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to get product ID");

        let response = client
            .get(format!("/api/produk/{}", product_id))
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<ProdukResponse> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        // Verify response structure for successful case
        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.data.is_some());

        let product = response_body.data.unwrap();
        assert!(product.id.is_some());
        assert!(!product.nama.is_empty());
        assert!(!product.kategori.is_empty());
        assert!(product.harga >= 0.0);
    }
}