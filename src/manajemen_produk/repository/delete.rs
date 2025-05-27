use crate::manajemen_produk::repository::dto::RepositoryError;
use sqlx::AnyPool;

pub async fn hapus_produk(pool: &AnyPool, id: i64) -> Result<bool, RepositoryError> {
    let result = sqlx::query("DELETE FROM produk WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}

pub async fn clear_all(pool: &AnyPool) -> Result<(), RepositoryError> {
    // Start transaction
    let mut tx = pool.begin().await?;
    
    // Clear all products
    sqlx::query("DELETE FROM produk")
        .execute(&mut *tx)
        .await?;
    
    // Reset auto increment counter (SQLite way)
    // For SQLite, we reset the sqlite_sequence table
    sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'produk'")
        .execute(&mut *tx)
        .await
        .ok(); // Ignore error if sqlite_sequence doesn't exist or no auto increment
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(())
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
        let db_pool = setup_test_db().await;
        
        // Insert a test product first
        let product_id = insert_test_product(&db_pool, "Test Laptop", "Elektronik", 10000000.0, 5).await;
        
        // Delete the product
        let result = hapus_produk(&db_pool, product_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap() == true); // Should return true for successful deletion
        
        // Verify the product was actually deleted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk WHERE id = $1")
            .bind(product_id)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products");
            
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_hapus_produk_nonexistent() {
        let db_pool = setup_test_db().await;
        
        // Try to delete a product that doesn't exist
        let result = hapus_produk(&db_pool, 999999).await;
        assert!(result.is_ok());
        assert!(result.unwrap() == false); // Should return false for non-existent product
    }

    #[tokio::test]
    async fn test_hapus_produk_negative_id() {
        let db_pool = setup_test_db().await;
        
        // Try to delete with negative ID
        let result = hapus_produk(&db_pool, -1).await;
        assert!(result.is_ok());
        assert!(result.unwrap() == false); // Should return false for invalid ID
    }

    #[tokio::test]
    async fn test_hapus_multiple_products() {
        let db_pool = setup_test_db().await;
        
        // Insert multiple test products
        let id1 = insert_test_product(&db_pool, "Product 1", "Kategori 1", 100000.0, 10).await;
        let id2 = insert_test_product(&db_pool, "Product 2", "Kategori 2", 200000.0, 20).await;
        let id3 = insert_test_product(&db_pool, "Product 3", "Kategori 3", 300000.0, 30).await;
        
        // Delete first product
        let result1 = hapus_produk(&db_pool, id1).await;
        assert!(result1.is_ok());
        assert!(result1.unwrap() == true);
        
        // Delete third product
        let result3 = hapus_produk(&db_pool, id3).await;
        assert!(result3.is_ok());
        assert!(result3.unwrap() == true);
        
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
        let db_pool = setup_test_db().await;
        
        // Insert a test product
        let product_id = insert_test_product(&db_pool, "Double Delete Test", "Test", 50000.0, 1).await;
        
        // Delete the product first time
        let result1 = hapus_produk(&db_pool, product_id).await;
        assert!(result1.is_ok());
        assert!(result1.unwrap() == true);
        
        // Try to delete the same product again
        let result2 = hapus_produk(&db_pool, product_id).await;
        assert!(result2.is_ok());
        assert!(result2.unwrap() == false); // Should return false as product no longer exists
    }

    #[tokio::test]
    async fn test_clear_all_empty_table() {
        let db_pool = setup_test_db().await;
        
        // Clear empty table (should not fail)
        let result = clear_all(&db_pool).await;
        assert!(result.is_ok()); // Should succeed even on empty table
        
        // Verify table is still empty
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count products");
            
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_clear_all_with_data() {
        let db_pool = setup_test_db().await;
        
        // Insert multiple test products
        insert_test_product(&db_pool, "Product A", "Category A", 1000.0, 1).await;
        insert_test_product(&db_pool, "Product B", "Category B", 2000.0, 2).await;
        insert_test_product(&db_pool, "Product C", "Category C", 3000.0, 3).await;
        
        // Verify products were inserted
        let initial_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count initial products");
            
        assert_eq!(initial_count, 3);
        
        // Clear all products
        let result = clear_all(&db_pool).await;
        assert!(result.is_ok()); // Should succeed
        
        // Verify all products were deleted
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM produk")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to count final products");
            
        assert_eq!(final_count, 0);
        
        // Test that auto increment is reset by inserting a new product
        let new_id = insert_test_product(&db_pool, "New Product", "New Category", 5000.0, 5).await;
        // After clear_all and sqlite_sequence reset, new ID should start from 1
        assert_eq!(new_id, 1);
    }

    #[tokio::test] 
    async fn test_hapus_produk_preserves_other_data() {
        let db_pool = setup_test_db().await;
        
        // Insert test products with specific data
        let id1 = insert_test_product(&db_pool, "Keep This", "Electronics", 500000.0, 15).await;
        let id2 = insert_test_product(&db_pool, "Delete This", "Books", 25000.0, 100).await;
        let id3 = insert_test_product(&db_pool, "Also Keep", "Clothing", 150000.0, 50).await;
        
        // Delete middle product
        let result = hapus_produk(&db_pool, id2).await;
        assert!(result.is_ok());
        assert!(result.unwrap() == true);
        
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
    async fn test_hapus_produk_zero_id() {
        let db_pool = setup_test_db().await;
        
        // Try to delete with ID 0
        let result = hapus_produk(&db_pool, 0).await;
        assert!(result.is_ok());
        assert!(result.unwrap() == false); // Should return false for ID 0
    }
}