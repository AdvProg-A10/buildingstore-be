use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{validate_produk, RepositoryError};
use sqlx::AnyPool;

pub async fn update_produk(pool: &AnyPool, id: i64, produk: &Produk) -> Result<bool, RepositoryError> {
    // Validasi input
    validate_produk(produk)?;
    
    let result = sqlx::query(
        r#"
        UPDATE produk 
        SET nama = $1, kategori = $2, harga = $3, stok = $4, deskripsi = $5
        WHERE id = $6
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i32)
    .bind(&produk.deskripsi)
    .bind(id)
    .execute(pool)
    .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_stok(pool: &AnyPool, id: i64, new_stok: u32) -> Result<bool, RepositoryError> {
    let result = sqlx::query("UPDATE produk SET stok = $1 WHERE id = $2")
        .bind(new_stok as i32)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_harga(pool: &AnyPool, id: i64, new_harga: f64) -> Result<bool, RepositoryError> {
    if new_harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    let result = sqlx::query("UPDATE produk SET harga = $1 WHERE id = $2")
        .bind(new_harga)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
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

    #[tokio::test]
    async fn test_update_produk_valid() {
        let db_pool = setup_test_db().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let updated_produk = Produk {
            id: Some(product_id),
            nama: "Updated Laptop".to_string(),
            kategori: "Elektronik".to_string(),
            harga: 15000000.99,
            stok: 25,
            deskripsi: Some("Updated description for laptop".to_string()),
        };

        let result = update_produk(&db_pool, product_id, &updated_produk).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Verify the update
        let row = sqlx::query("SELECT * FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch updated product");

        let nama: String = row.get("nama");
        let kategori: String = row.get("kategori");
        let harga: f64 = row.get("harga");
        let stok: i32 = row.get("stok");
        let deskripsi: Option<String> = row.try_get("deskripsi").unwrap_or(None);
        
        assert_eq!(nama, "Updated Laptop");
        assert_eq!(kategori, "Elektronik");
        assert!((harga - 15000000.99).abs() < f64::EPSILON);
        assert_eq!(stok, 25);
        assert_eq!(deskripsi, Some("Updated description for laptop".to_string()));
    }

    #[tokio::test]
    async fn test_update_produk_not_found() {
        let db_pool = setup_test_db().await;
        
        let produk = Produk {
            id: Some(999),
            nama: "Non-existent Product".to_string(),
            kategori: "Test".to_string(),
            harga: 100000.0,
            stok: 10,
            deskripsi: None,
        };

        let result = update_produk(&db_pool, 999, &produk).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RepositoryError::NotFound => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_stok_valid() {
        let db_pool = setup_test_db().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let new_stok = 100u32;
        let result = update_stok(&db_pool, product_id, new_stok).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Verify the stock update
        let row = sqlx::query("SELECT stok FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch updated product");

        let stok: i32 = row.get("stok");
        assert_eq!(stok, 100);
    }

    #[tokio::test]
    async fn test_update_stok_not_found() {
        let db_pool = setup_test_db().await;
        
        let result = update_stok(&db_pool, 999, 100).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RepositoryError::NotFound => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_harga_valid() {
        let db_pool = setup_test_db().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let new_harga = 250000.75;
        let result = update_harga(&db_pool, product_id, new_harga).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Verify the price update
        let row = sqlx::query("SELECT harga FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch updated product");

        let harga: f64 = row.get("harga");
        assert!((harga - 250000.75).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_update_harga_negative_price() {
        let db_pool = setup_test_db().await;
        let product_id = insert_test_produk(&db_pool).await;
        
        let result = update_harga(&db_pool, product_id, -100.0).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RepositoryError::ValidationError(msg) => {
                assert_eq!(msg, "Harga tidak boleh negatif");
            },
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_update_harga_not_found() {
        let db_pool = setup_test_db().await;
        
        let result = update_harga(&db_pool, 999, 100000.0).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RepositoryError::NotFound => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }
}