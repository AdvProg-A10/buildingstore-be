use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{RepositoryError};
use sqlx::{AnyPool, Row};

// SOLUSI 1: Menggunakan try_get dengan handling NULL secara manual
pub async fn ambil_semua_produk(pool: &AnyPool) -> Result<Vec<Produk>, RepositoryError> {
    let rows = sqlx::query("SELECT id, nama, kategori, CAST(harga as DOUBLE PRECISION) as harga, stok, deskripsi FROM produk ORDER BY id")
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        // Handle nullable deskripsi dengan cara yang lebih aman
        let deskripsi: Option<String> = match row.try_get("deskripsi") {
            Ok(val) => val,
            Err(_) => None, // Jika error (termasuk NULL), set ke None
        };
        
        products.push(Produk::with_id(
            row.try_get("id")?,
            row.try_get("nama")?,
            row.try_get("kategori")?,
            row.try_get("harga")?,
            row.try_get::<i32, _>("stok")? as u32,
            deskripsi,
        ));
    }
    
    Ok(products)
}

pub async fn ambil_produk_by_id(pool: &AnyPool, id: i64) -> Result<Option<Produk>, RepositoryError> {
    let row = sqlx::query("SELECT id, nama, kategori, CAST(harga as DOUBLE PRECISION) as harga, stok, deskripsi FROM produk WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(row) => {
            // Handle nullable deskripsi dengan cara yang lebih aman
            let deskripsi: Option<String> = match row.try_get("deskripsi") {
                Ok(val) => val,
                Err(_) => None, // Jika error (termasuk NULL), set ke None
            };
            
            Ok(Some(Produk::with_id(
                row.try_get("id")?,
                row.try_get("nama")?,
                row.try_get("kategori")?,
                row.try_get("harga")?,
                row.try_get::<i32, _>("stok")? as u32,
                deskripsi,
            )))
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}, Row};
    use crate::manajemen_produk::model::Produk;

    async fn setup_test_db() -> sqlx::Pool<sqlx::Any> {
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

    async fn insert_test_data(pool: &sqlx::Pool<sqlx::Any>) {
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

    #[tokio::test]
    async fn test_ambil_semua_produk_with_data() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        let result = ambil_semua_produk(&db_pool).await;
        assert!(result.is_ok(), "Query should succeed: {:?}", result.err());

        let products = result.unwrap();
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
    async fn test_ambil_semua_produk_empty_table() {
        let db_pool = setup_test_db().await;
        // Don't insert any data

        let result = ambil_semua_produk(&db_pool).await;
        assert!(result.is_ok());

        let products = result.unwrap();
        assert_eq!(products.len(), 0);
        assert!(products.is_empty());
    }

    #[tokio::test]
    async fn test_ambil_semua_produk_order_by_id() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        let result = ambil_semua_produk(&db_pool).await;
        assert!(result.is_ok(), "Query should succeed: {:?}", result.err());

        let products = result.unwrap();
        
        // Verify products are ordered by ID
        for i in 1..products.len() {
            if let (Some(prev_id), Some(curr_id)) = (products[i-1].id, products[i].id) {
                assert!(prev_id < curr_id, "Products should be ordered by ID");
            }
        }
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id_existing() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        // Get the first product ID
        let all_products = ambil_semua_produk(&db_pool).await.unwrap();
        let first_product_id = all_products[0].id.unwrap();

        let result = ambil_produk_by_id(&db_pool, first_product_id).await;
        assert!(result.is_ok(), "Query should succeed: {:?}", result.err());

        let product = result.unwrap();
        assert!(product.is_some());

        let found_product = product.unwrap();
        assert_eq!(found_product.id, Some(first_product_id));
        assert_eq!(found_product.nama, "Laptop Gaming");
        assert_eq!(found_product.kategori, "Elektronik");
        assert!((found_product.harga - 15000000.50).abs() < f64::EPSILON);
        assert_eq!(found_product.stok, 10);
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id_nonexistent() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        // Try to get a product with ID that doesn't exist
        let result = ambil_produk_by_id(&db_pool, 999999).await;
        assert!(result.is_ok());

        let product = result.unwrap();
        assert!(product.is_none());
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id_zero_stock() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        // Find the keyboard with zero stock
        let all_products = ambil_semua_produk(&db_pool).await.unwrap();
        let keyboard_product = all_products.iter()
            .find(|p| p.nama == "Keyboard Mechanical")
            .unwrap();
        let keyboard_id = keyboard_product.id.unwrap();

        let result = ambil_produk_by_id(&db_pool, keyboard_id).await;
        assert!(result.is_ok(), "Query should succeed: {:?}", result.err());

        let product = result.unwrap();
        assert!(product.is_some());

        let found_product = product.unwrap();
        assert_eq!(found_product.nama, "Keyboard Mechanical");
        assert_eq!(found_product.stok, 0);
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id_with_null_description() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        // Find the mouse with null description
        let all_products = ambil_semua_produk(&db_pool).await.unwrap();
        let mouse_product = all_products.iter()
            .find(|p| p.nama == "Mouse Wireless")
            .unwrap();
        let mouse_id = mouse_product.id.unwrap();

        let result = ambil_produk_by_id(&db_pool, mouse_id).await;
        assert!(result.is_ok(), "Query should succeed: {:?}", result.err());

        let product = result.unwrap();
        assert!(product.is_some());

        let found_product = product.unwrap();
        assert_eq!(found_product.nama, "Mouse Wireless");
        assert_eq!(found_product.deskripsi, None);
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id_negative_id() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        // Try to get a product with negative ID
        let result = ambil_produk_by_id(&db_pool, -1).await;
        assert!(result.is_ok());

        let product = result.unwrap();
        assert!(product.is_none());
    }

    #[tokio::test]
    async fn test_ambil_semua_produk_with_special_characters() {
        let db_pool = setup_test_db().await;
        
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

        let result = ambil_semua_produk(&db_pool).await;
        assert!(result.is_ok());

        let products = result.unwrap();
        assert_eq!(products.len(), 1);

        let product = &products[0];
        assert_eq!(product.nama, "Café Latte & Cappuccino™");
        assert_eq!(product.kategori, "Minuman & Makanan");
        assert_eq!(product.deskripsi, Some("Premium coffee blend with special ingredients: açaí, ginseng & organic milk".to_string()));
    }

    #[tokio::test]
    async fn test_ambil_semua_produk_with_large_values() {
        let db_pool = setup_test_db().await;
        
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

        let result = ambil_semua_produk(&db_pool).await;
        assert!(result.is_ok());

        let products = result.unwrap();
        assert_eq!(products.len(), 1);

        let product = &products[0];
        assert_eq!(product.nama, "Server Enterprise");
        assert!((product.harga - 999999999.99).abs() < 1.0); // Floating point comparison
        assert_eq!(product.stok, 999999);
    }

    #[tokio::test]
    async fn test_multiple_queries_consistency() {
        let db_pool = setup_test_db().await;
        insert_test_data(&db_pool).await;

        // Get all products
        let all_products = ambil_semua_produk(&db_pool).await.unwrap();
        assert_eq!(all_products.len(), 5);

        // Get each product individually and compare
        for product in &all_products {
            let individual_result = ambil_produk_by_id(&db_pool, product.id.unwrap()).await;
            
            assert!(individual_result.is_ok(), "Individual query should succeed: {:?}", individual_result.err());
            assert!(individual_result.as_ref().unwrap().is_some());
            
            let individual_product = individual_result.unwrap().unwrap();
            assert_eq!(individual_product.id, product.id);
            assert_eq!(individual_product.nama, product.nama);
            assert_eq!(individual_product.kategori, product.kategori);
            assert!((individual_product.harga - product.harga).abs() < f64::EPSILON);
            assert_eq!(individual_product.stok, product.stok);
            assert_eq!(individual_product.deskripsi, product.deskripsi);
        }
    }
}