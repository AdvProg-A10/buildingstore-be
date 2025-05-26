use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{validate_produk, RepositoryError};
use sqlx::{AnyPool, Row};

pub async fn tambah_produk(pool: &AnyPool, produk: &Produk) -> Result<i64, RepositoryError> {
    // Validasi terlebih dahulu
    validate_produk(produk)?;
    
    let result = sqlx::query(
        r#"
        INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i32)
    .bind(&produk.deskripsi)
    .fetch_one(pool)
    .await?;
    
    Ok(result.get("id"))
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

    #[tokio::test]
    async fn test_tambah_produk_valid() {
        let db_pool = setup_test_db().await;
        
        let produk = Produk {
            id: None,
            nama: "Laptop Gaming".to_string(),
            kategori: "Elektronik".to_string(),
            harga: 15000000.50, // Menggunakan f64 langsung
            stok: 10,
            deskripsi: Some("Laptop gaming high-end dengan RTX 4080".to_string()),
        };

        let result = tambah_produk(&db_pool, &produk).await;
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        assert!(product_id > 0);

        // Verify the product was actually inserted
        let row = sqlx::query("SELECT * FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch inserted product");

        let nama: String = row.get("nama");
        let kategori: String = row.get("kategori");
        let stok: i32 = row.get("stok");
        let harga: f64 = row.get("harga");
        
        assert_eq!(nama, "Laptop Gaming");
        assert_eq!(kategori, "Elektronik");
        assert_eq!(stok, 10);
        assert!((harga - 15000000.50).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_tambah_produk_with_minimal_data() {
        let db_pool = setup_test_db().await;
        
        let produk = Produk {
            id: None,
            nama: "Mouse".to_string(),
            kategori: "Aksesoris".to_string(),
            harga: 150000.00, // f64
            stok: 50,
            deskripsi: None, // No description
        };

        let result = tambah_produk(&db_pool, &produk).await;
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        assert!(product_id > 0);

        // Verify insertion
        let row = sqlx::query("SELECT nama, kategori, deskripsi FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch inserted product");

        let nama: String = row.get("nama");
        let kategori: String = row.get("kategori");
        let deskripsi: Option<String> = row.get("deskripsi");
        
        assert_eq!(nama, "Mouse");
        assert_eq!(kategori, "Aksesoris");
        assert!(deskripsi.is_none());
    }

    #[tokio::test]
    async fn test_tambah_produk_with_zero_stock() {
        let db_pool = setup_test_db().await;
        
        let produk = Produk {
            id: None,
            nama: "Keyboard Mechanical".to_string(),
            kategori: "Aksesoris".to_string(),
            harga: 750000.99, // f64
            stok: 0, // Zero stock
            deskripsi: Some("Keyboard mechanical blue switch".to_string()),
        };

        let result = tambah_produk(&db_pool, &produk).await;
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let row = sqlx::query("SELECT stok FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch inserted product");

        let stok: i32 = row.get("stok");
        assert_eq!(stok, 0);
    }

    #[tokio::test]
    async fn test_tambah_produk_with_large_values() {
        let db_pool = setup_test_db().await;
        
        let produk = Produk {
            id: None,
            nama: "Server Enterprise".to_string(),
            kategori: "Server".to_string(),
            harga: 999999999.99, // f64 - Large price
            stok: 999999, // Large stock
            deskripsi: Some("High-end enterprise server with redundant systems and 24/7 support warranty".to_string()),
        };

        let result = tambah_produk(&db_pool, &produk).await;
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let row = sqlx::query("SELECT harga, stok FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch inserted product");

        let stok: i32 = row.get("stok");
        let harga: f64 = row.get("harga");
        assert_eq!(stok, 999999);
        assert!((harga - 999999999.99).abs() < 1.0); // Floating point comparison
    }

    #[tokio::test]
    async fn test_tambah_multiple_produk() {
        let db_pool = setup_test_db().await;
        
        let produk1 = Produk {
            id: None,
            nama: "iPhone 15".to_string(),
            kategori: "Smartphone".to_string(),
            harga: 15000000.00, // f64
            stok: 25,
            deskripsi: Some("Latest iPhone model".to_string()),
        };

        let produk2 = Produk {
            id: None,
            nama: "Samsung Galaxy S24".to_string(),
            kategori: "Smartphone".to_string(),
            harga: 12000000.00, // f64
            stok: 30,
            deskripsi: Some("Latest Samsung flagship".to_string()),
        };

        let result1 = tambah_produk(&db_pool, &produk1).await;
        let result2 = tambah_produk(&db_pool, &produk2).await;
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let id1 = result1.unwrap();
        let id2 = result2.unwrap();
        
        // Ensure different IDs
        assert_ne!(id1, id2);
        
        // Verify both products exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products");
            
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_tambah_produk_with_special_characters() {
        let db_pool = setup_test_db().await;
        
        let produk = Produk {
            id: None,
            nama: "Café Latte & Cappuccino™".to_string(),
            kategori: "Minuman & Makanan".to_string(),
            harga: 45000.50, // f64
            stok: 100,
            deskripsi: Some("Premium coffee blend with special ingredients: açaí, ginseng & organic milk".to_string()),
        };

        let result = tambah_produk(&db_pool, &produk).await;
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let row = sqlx::query("SELECT nama, kategori FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch inserted product");

        let nama: String = row.get("nama");
        let kategori: String = row.get("kategori");
        
        assert_eq!(nama, "Café Latte & Cappuccino™");
        assert_eq!(kategori, "Minuman & Makanan");
    }
}