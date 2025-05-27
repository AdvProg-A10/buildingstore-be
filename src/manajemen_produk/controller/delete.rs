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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}, AnyPool, Row};
    use crate::manajemen_produk::controller::dto::ApiResponse;

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
            .mount("/api", routes![hapus_produk]);
            
        let client = Client::tracked(rocket)
            .await
            .expect("Valid rocket instance");
            
        (client, db_pool)
    }

    async fn insert_test_product(pool: &AnyPool, nama: &str, kategori: &str, harga: f64, stok: i32) -> i64 {
        let result = sqlx::query(
            r#"
            INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#
        )
        .bind(nama)
        .bind(kategori)
        .bind(harga)
        .bind(stok)
        .bind("Test product description")
        .fetch_one(pool)
        .await
        .expect("Failed to insert test product");
        
        result.get("id")
    }

    #[tokio::test]
    async fn test_hapus_produk_existing() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert a test product first
        let product_id = insert_test_product(&db_pool, "Test Laptop", "Elektronik", 10000000.0, 5).await;
        
        // Delete the product via controller
        let response = client
            .delete(format!("/api/produk/{}", product_id))
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<()> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.message.unwrap().contains("berhasil dihapus"));
        assert!(response_body.data.is_none());
        
        // Verify the product was actually deleted from database
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products");
            
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_hapus_produk_nonexistent() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        // Try to delete a product that doesn't exist
        let response = client
            .delete("/api/produk/999999")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<()> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.message.unwrap().contains("tidak ditemukan"));
        assert!(response_body.data.is_none());
    }

    #[tokio::test]
    async fn test_hapus_produk_negative_id() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        // Try to delete with negative ID
        let response = client
            .delete("/api/produk/-1")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<()> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.message.unwrap().contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn test_hapus_produk_zero_id() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        // Try to delete with ID 0
        let response = client
            .delete("/api/produk/0")
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<()> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.is_some());
        assert!(response_body.message.unwrap().contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn test_hapus_multiple_products_sequentially() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert multiple test products
        let id1 = insert_test_product(&db_pool, "Product 1", "Kategori 1", 100000.0, 10).await;
        let id2 = insert_test_product(&db_pool, "Product 2", "Kategori 2", 200000.0, 20).await;
        let id3 = insert_test_product(&db_pool, "Product 3", "Kategori 3", 300000.0, 30).await;
        
        // Delete first product
        let response1 = client
            .delete(format!("/api/produk/{}", id1))
            .dispatch()
            .await;
        assert_eq!(response1.status(), rocket::http::Status::Ok);
        
        let response_body1: ApiResponse<()> = response1.into_json().await.expect("Valid JSON");
        assert!(response_body1.success);
        
        // Delete third product
        let response3 = client
            .delete(format!("/api/produk/{}", id3))
            .dispatch()
            .await;
        assert_eq!(response3.status(), rocket::http::Status::Ok);
        
        let response_body3: ApiResponse<()> = response3.into_json().await.expect("Valid JSON");
        assert!(response_body3.success);
        
        // Verify only second product remains
        let remaining_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count remaining products");
            
        assert_eq!(remaining_count, 1);
        
        // Verify the remaining product is the second one
        let remaining_row = sqlx::query("SELECT nama FROM produk WHERE id = $1")
            .bind(id2)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch remaining product");
            
        let nama: String = remaining_row.get("nama");
        assert_eq!(nama, "Product 2");
    }

    #[tokio::test]
    async fn test_hapus_produk_twice() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert a test product
        let product_id = insert_test_product(&db_pool, "Double Delete Test", "Test", 50000.0, 1).await;
        
        // Delete the product first time
        let response1 = client
            .delete(format!("/api/produk/{}", product_id))
            .dispatch()
            .await;
        assert_eq!(response1.status(), rocket::http::Status::Ok);
        
        let response_body1: ApiResponse<()> = response1.into_json().await.expect("Valid JSON");
        assert!(response_body1.success);
        
        // Try to delete the same product again
        let response2 = client
            .delete(format!("/api/produk/{}", product_id))
            .dispatch()
            .await;
        assert_eq!(response2.status(), rocket::http::Status::Ok);
        
        let response_body2: ApiResponse<()> = response2.into_json().await.expect("Valid JSON");
        assert!(!response_body2.success); // Should return false/not found
        assert!(response_body2.message.unwrap().contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn test_hapus_produk_preserves_other_data() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert test products with specific data
        let id1 = insert_test_product(&db_pool, "Keep This", "Electronics", 500000.0, 15).await;
        let id2 = insert_test_product(&db_pool, "Delete This", "Books", 25000.0, 100).await;
        let id3 = insert_test_product(&db_pool, "Also Keep", "Clothing", 150000.0, 50).await;
        
        // Delete middle product via controller
        let response = client
            .delete(format!("/api/produk/{}", id2))
            .dispatch()
            .await;
        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<()> = response.into_json().await.expect("Valid JSON");
        assert!(response_body.success);
        
        // Verify other products still exist with correct data
        let row1 = sqlx::query("SELECT nama, kategori, harga, stok FROM produk WHERE id = $1")
            .bind(id1)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch first product");
            
        let row3 = sqlx::query("SELECT nama, kategori, harga, stok FROM produk WHERE id = $1")
            .bind(id3)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch third product");
        
        // Check first product data
        let nama1: String = row1.get("nama");
        let kategori1: String = row1.get("kategori");
        let harga1: f64 = row1.get("harga");
        let stok1: i32 = row1.get("stok");
        
        assert_eq!(nama1, "Keep This");
        assert_eq!(kategori1, "Electronics");
        assert!((harga1 - 500000.0).abs() < f64::EPSILON);
        assert_eq!(stok1, 15);
        
        // Check third product data
        let nama3: String = row3.get("nama");
        let kategori3: String = row3.get("kategori");
        let harga3: f64 = row3.get("harga");
        let stok3: i32 = row3.get("stok");
        
        assert_eq!(nama3, "Also Keep");
        assert_eq!(kategori3, "Clothing");
        assert!((harga3 - 150000.0).abs() < f64::EPSILON);
        assert_eq!(stok3, 50);
        
        // Verify deleted product is gone
        let deleted_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk WHERE id = $1")
            .bind(id2)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count deleted product");
            
        assert_eq!(deleted_count, 0);
    }

    #[tokio::test]
    async fn test_hapus_produk_large_id() {
        let (client, _db_pool) = setup_rocket_client().await;
        
        // Try to delete with very large ID
        let response = client
            .delete("/api/produk/9223372036854775807") // i64::MAX
            .dispatch()
            .await;

        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let response_body: ApiResponse<()> = response
            .into_json()
            .await
            .expect("Valid JSON response");

        assert!(!response_body.success);
        assert!(response_body.message.unwrap().contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn test_hapus_all_products_one_by_one() {
        let (client, db_pool) = setup_rocket_client().await;
        
        // Insert multiple test products
        let mut product_ids = Vec::new();
        for i in 1..=5 {
            let id = insert_test_product(
                &db_pool, 
                &format!("Product {}", i), 
                &format!("Category {}", i), 
                (i as f64) * 100000.0, 
                i * 10
            ).await;
            product_ids.push(id);
        }
        
        // Verify all products exist
        let initial_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products");
        assert_eq!(initial_count, 5);
        
        // Delete all products one by one
        for (index, product_id) in product_ids.iter().enumerate() {
            let response = client
                .delete(format!("/api/produk/{}", product_id))
                .dispatch()
                .await;
                
            assert_eq!(response.status(), rocket::http::Status::Ok);
            
            let response_body: ApiResponse<()> = response.into_json().await.expect("Valid JSON");
            assert!(response_body.success);
            
            // Check remaining count
            let remaining_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
                .fetch_one(&db_pool)
                .await
                .expect("Failed to count remaining products");
            assert_eq!(remaining_count, (4 - index) as i64);
        }
        
        // Verify no products remain
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products after deletion");
        assert_eq!(final_count, 0);
    }
}