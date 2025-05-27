use sqlx::{Any, pool::PoolConnection, any::AnyRow, Row};
use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;

use async_trait::async_trait;

pub struct SupplierRepositoryImpl;

impl SupplierRepositoryImpl {
    pub fn new() -> Self {
        Self
    }

    fn parse_row_to_supplier(row: AnyRow) -> Result<Supplier, sqlx::Error> {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let jenis_barang: String = row.get("jenis_barang");
        let jumlah_barang: i32 = row.get("jumlah_barang");
        let resi: String = row.get("resi");
        let updated_at: String = row.get("updated_at");

        Ok(Supplier {
            id,
            name,
            jenis_barang,
            jumlah_barang,
            resi,
            updated_at,
        })
    }
}

#[async_trait]
impl SupplierRepository for SupplierRepositoryImpl {
    async fn save(&self, supplier: Supplier, mut db: PoolConnection<Any>) -> Result<Supplier, sqlx::Error> {
        let query = "
            INSERT INTO suppliers (id, name, jenis_barang, jumlah_barang, resi, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
        ";

        sqlx::query(query)
            .bind(&supplier.id)
            .bind(&supplier.name)
            .bind(&supplier.jenis_barang)
            .bind(supplier.jumlah_barang)
            .bind(&supplier.resi)
            .bind(&supplier.updated_at)
            .execute(&mut *db)
            .await?;

        Ok(supplier)
    }

    async fn find_by_id(&self, id: &str, mut db: PoolConnection<Any>) -> Result<Supplier, sqlx::Error> {
        let query = "SELECT * FROM suppliers WHERE id = $1";

        let row = sqlx::query(query)
            .bind(id)
            .fetch_one(&mut *db)
            .await?;

        Self::parse_row_to_supplier(row)
    }

    async fn update(&self, supplier: Supplier, mut db: PoolConnection<Any>) -> Result<(), sqlx::Error> {
        let query = "
            UPDATE suppliers
            SET name = $1,
                jenis_barang = $2,
                jumlah_barang = $3,
                resi = $4,
                updated_at = $5
            WHERE id = $6
        ";

        let result = sqlx::query(query)
            .bind(&supplier.name)
            .bind(&supplier.jenis_barang)
            .bind(supplier.jumlah_barang)
            .bind(&supplier.resi)
            .bind(supplier.updated_at)
            .bind(&supplier.id)
            .execute(&mut *db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn delete(&self, id: &str, mut db: PoolConnection<Any>) -> Result<(), sqlx::Error> {
        let query = "DELETE FROM suppliers WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id)
            .execute(&mut *db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn find_all(&self, mut db: PoolConnection<Any>) -> Result<Vec<Supplier>, sqlx::Error> {
        let query = "SELECT * FROM suppliers";
        let rows = sqlx::query(query)
            .fetch_all(&mut *db)
            .await?;

        let mut suppliers = Vec::new();
        for row in rows {
            suppliers.push(Self::parse_row_to_supplier(row)?);
        }
        Ok(suppliers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}};
    use chrono::Utc;
    use uuid::Uuid;

    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;

    async fn setup_repository() -> (SupplierRepositoryImpl, sqlx::Pool<Any>) {
        install_default_drivers();
        let db_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test DB");

        sqlx::migrate!("migrations/test")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        (SupplierRepositoryImpl::new(), db_pool)
    }

    #[tokio::test]
    async fn test_save_supplier() {
        let (repository, db_pool) = setup_repository().await;
        let supplier_id = format!("SUP-{}", Uuid::new_v4());

        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.save(supplier.clone(), db_conn).await;
        assert!(result.is_ok());

        let saved_supplier = result.unwrap();
        assert_eq!(saved_supplier.id, supplier_id);
        assert_eq!(saved_supplier.name, "PT. Ayam");
    }

#[tokio::test]
async fn test_find_supplier_by_id() {
    let (repository, db_pool) = setup_repository().await;
    let supplier_id = format!("SUP-{}", Uuid::new_v4());

    let supplier = Supplier {
        id: supplier_id.clone(),
        name: "PT. Ayam".to_string(),
        jenis_barang: "ayam".to_string(),
        jumlah_barang: 1000,
        resi: "2306206282".to_string(),
        updated_at: Utc::now().to_rfc3339(),
    };

    let db_conn = db_pool.acquire().await.unwrap();
    repository.save(supplier.clone(), db_conn).await.unwrap();

    let db_conn = db_pool.acquire().await.unwrap();
    let result = repository.find_by_id(&supplier_id, db_conn).await;

    let found_supplier = result.expect(&format!(
        "Failed to find supplier by ID '{}', which was expected to exist.",
        supplier_id
    ));
    assert_eq!(found_supplier.id, supplier_id);
}

    #[tokio::test]
    async fn test_find_nonexistent_supplier() {
        let (repository, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.find_by_id("non-existent-id", db_conn).await;
    
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {},
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_supplier() {
        let (repository, db_pool) = setup_repository().await;
        let supplier_id = format!("SUP-{}", Uuid::new_v4());

        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(supplier.clone(), db_conn).await.unwrap();

        let mut updated_supplier = supplier.clone();
        updated_supplier.jumlah_barang = 1;
        updated_supplier.updated_at = Utc::now().to_rfc3339();
        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.update(updated_supplier, db_conn).await;

        assert!(result.is_ok());
        let db_conn = db_pool.acquire().await.unwrap();
        let found_supplier = repository.find_by_id(&supplier_id, db_conn).await.unwrap();
        assert_eq!(found_supplier.jumlah_barang, 1);
    }

    #[tokio::test]
    async fn test_delete_supplier() {
        let (repository, db_pool) = setup_repository().await;
        let supplier_id = format!("SUP-{}", Uuid::new_v4());

        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(supplier.clone(), db_conn).await.unwrap();

        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.delete(&supplier_id, db_conn).await;

        assert!(result.is_ok());
        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.find_by_id(&supplier_id, db_conn).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {},
            _ => panic!("Expected RowNotFound error"),
        }
    }

        #[tokio::test]
    async fn test_find_all_suppliers_empty() {
        let (repository, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.find_all(db_conn).await;
        
        assert!(result.is_ok());
        let suppliers = result.unwrap();
        assert!(suppliers.is_empty());
    }

    #[tokio::test]
    async fn test_find_all_suppliers_multiple() {
        let (repository, db_pool) = setup_repository().await;

        let supplier1 = Supplier {
            id: format!("SUP-{}", Uuid::new_v4()),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let supplier2 = Supplier {
            id: format!("SUP-{}", Uuid::new_v4()),
            name: "PT. Sapi".to_string(),
            jenis_barang: "sapi".to_string(),
            jumlah_barang: 500,
            resi: "2306206283".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(supplier1.clone(), db_conn).await.unwrap();
        
        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(supplier2.clone(), db_conn).await.unwrap();

        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.find_all(db_conn).await;
        
        assert!(result.is_ok());
        let suppliers = result.unwrap();
        assert_eq!(suppliers.len(), 2);
        
        let ids: Vec<String> = suppliers.iter().map(|s| s.id.clone()).collect();
        assert!(ids.contains(&supplier1.id));
        assert!(ids.contains(&supplier2.id));
    }

    #[tokio::test]
    async fn test_update_nonexistent_supplier() {
        let (repository, db_pool) = setup_repository().await;
        
        let nonexistent_supplier = Supplier {
            id: format!("SUP-{}", Uuid::new_v4()),
            name: "Non-existent".to_string(),
            jenis_barang: "none".to_string(),
            jumlah_barang: 0,
            resi: "000".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.update(nonexistent_supplier, db_conn).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {},
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_nonexistent_supplier() {
        let (repository, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.delete("non-existent-id", db_conn).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {},
            _ => panic!("Expected RowNotFound error"),
        }
    }
}