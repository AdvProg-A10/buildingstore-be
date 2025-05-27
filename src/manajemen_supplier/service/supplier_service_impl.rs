use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Any, Pool, Error as SqlxError};
use chrono::Utc; 
use uuid::Uuid; 

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;

pub struct SupplierServiceImpl {
    supplier_repo: Arc<dyn SupplierRepository>,
    transaction_repo: Arc<dyn SupplierTransactionRepository>,
    dispatcher: Arc<dyn SupplierNotifier>,
}

impl SupplierServiceImpl {
    pub fn new(
        supplier_repo: Arc<dyn SupplierRepository>,
        transaction_repo: Arc<dyn SupplierTransactionRepository>,
        dispatcher: Arc<dyn SupplierNotifier>,
    ) -> Self {
        Self { supplier_repo, dispatcher, transaction_repo }
    }
}

#[async_trait]
impl SupplierService for SupplierServiceImpl {
    async fn save_supplier(
        &self,
        db_pool: Pool<Any>,
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
    ) -> Result<Supplier, String> {
        let conn = db_pool.acquire().await
            .map_err(|e| format!("Service: Failed to acquire DB connection: {}", e))?;
        
        let supplier_to_save = Supplier {
            id: Uuid::new_v4().to_string(), 
            name,
            jenis_barang,
            jumlah_barang,
            resi,
            updated_at: Utc::now().to_rfc3339(), 
        };
        
        let saved_supplier = self.supplier_repo.save(supplier_to_save, conn).await
            .map_err(|e| format!("Service: Repository save error: {}", e))?;

        self.dispatcher.notify_supplier_saved(&saved_supplier).await;
        Ok(saved_supplier)
    }

    async fn update_supplier(
        &self,
        db_pool: Pool<Any>,
        id: String,
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
    ) -> Result<(), String> { 
        let conn = db_pool.acquire().await
            .map_err(|e| format!("Service: Failed to acquire DB connection: {}", e))?;
        let supplier_to_update = Supplier {
            id,
            name,
            jenis_barang,
            jumlah_barang,
            resi,
            updated_at: Utc::now().to_rfc3339(), 
        };
        
        self.supplier_repo.update(supplier_to_update, conn).await
            .map_err(|e| match e {
                SqlxError::RowNotFound => "Service: Supplier not found for update.".to_string(),
                _ => format!("Service: Repository update error: {}", e),
            })
    }

    async fn delete_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<(), String> {
        let conn = db_pool.acquire().await
            .map_err(|e| format!("Service: Failed to acquire DB connection: {}", e))?;

        self.supplier_repo.delete(id, conn).await
            .map_err(|e| match e {
                SqlxError::RowNotFound => "Service: Supplier not found for delete.".to_string(),
                _ => format!("Service: Repository delete error: {}", e),
            })
    }

    async fn get_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<Option<Supplier>, String> {
        let conn = match db_pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                return Err(format!("Service: Failed to acquire DB connection: {e}"));
            }
        };
        match self.supplier_repo.find_by_id(id, conn).await {
            Ok(s) => Ok(Some(s)),
            Err(SqlxError::RowNotFound) => Ok(None),
            Err(e) => {
                return Err(format!("Service: Repository error: {e}"));
            }
        }
    }

    async fn get_all_suppliers(&self, db_pool: Pool<Any>) -> Result<Vec<Supplier>, String> {
        let conn = match db_pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                return Err(format!("Service: Failed to acquire DB connection: {e}"));
            }
        };

        match self.supplier_repo.find_all(conn).await {
            Ok(s) => Ok(s),
            Err(e) => {
                return Err(format!("Service: Repository error: {e}"));
            }
        }
    }

    async fn get_all_supplier_transactions(&self, db_pool: Pool<Any>) -> Result<Vec<SupplierTransaction>, String> {
        let conn = match db_pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                return Err(format!("Service: Failed to acquire DB connection: {e}"));
            }
        };

        match self.transaction_repo.find_all(conn).await {
            Ok(s) => Ok(s),
            Err(e) => {
                return Err(format!("Service: Repository error: {e}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_supplier::repository::supplier_repository::MockSupplierRepository;
    use crate::manajemen_supplier::repository::supplier_transaction_repository::MockSupplierTransactionRepository;
    use crate::manajemen_supplier::service::supplier_notifier::MockSupplierNotifier;
    use mockall::predicate::*;
    use sqlx::any::AnyPoolOptions;
    use sqlx::pool::PoolConnection;

    async fn create_dummy_pool() -> Pool<Any> {
        sqlx::any::install_default_drivers();
        AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create dummy pool for tests")
    }

    fn create_test_supplier(id: &str, name: &str) -> Supplier {
        Supplier {
            id: id.to_string(),
            name: name.to_string(),
            jenis_barang: "Test Jenis".to_string(),
            jumlah_barang: 10,
            resi: "Test Resi".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_save_supplier_success() {
        let mut mock_repo = MockSupplierRepository::new();
        let mut mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        let name = "Test Supplier".to_string();
        let jenis_barang = "Electronics".to_string();
        let jumlah_barang = 100;
        let resi = "RESI123".to_string();

        let name_cl = name.clone();
        let jenis_barang_cl = jenis_barang.clone();
        let jumlah_barang_cl = jumlah_barang;
        let resi_cl = resi.clone();

        mock_repo.expect_save()
            .withf(move |s: &Supplier, _conn: &PoolConnection<Any>| {
                s.name == name_cl && s.jenis_barang == jenis_barang_cl && s.jumlah_barang == jumlah_barang_cl && s.resi == resi_cl
            })
            .times(1)
            .returning(|supplier, _conn| Box::pin(async move { Ok(supplier) }));

        mock_notifier.expect_notify_supplier_saved()
            .times(1)
            .returning(|_s| Box::pin(async move { () }));

        let service = SupplierServiceImpl::new(
            Arc::new(mock_repo),
            Arc::new(mock_transaction_repo),
            Arc::new(mock_notifier),
        );
        let pool = create_dummy_pool().await;

        let result = service.save_supplier(pool, name.clone(), jenis_barang.clone(), jumlah_barang, resi.clone()).await;
        assert!(result.is_ok());
        let saved_supplier = result.unwrap();
        assert_eq!(saved_supplier.name, name);
        assert_eq!(saved_supplier.jenis_barang, jenis_barang);
        assert!(!saved_supplier.id.is_empty());
        assert!(!saved_supplier.updated_at.is_empty());
    }
    
    #[tokio::test]
    async fn test_get_supplier_success() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();
        let supplier_id = "sup1";
        let expected_supplier = create_test_supplier(supplier_id, "Supplier Found");

        let expected_supplier_cl = expected_supplier.clone();
        mock_repo.expect_find_by_id()
            .with(eq(supplier_id), always())
            .times(1)
            .returning(move |_, _| {
                let supplier = expected_supplier_cl.clone();
                Box::pin(async move { Ok(supplier) })
            });

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;

        let result = service.get_supplier(pool, supplier_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_supplier));
    }

    #[tokio::test]
    async fn test_get_supplier_not_found() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();
        let supplier_id = "sup-not-exist";

        mock_repo.expect_find_by_id()
            .with(eq(supplier_id), always())
            .times(1)
            .returning(|_, _| Box::pin(async { Err(SqlxError::RowNotFound) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;

        let result = service.get_supplier(pool, supplier_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_get_all_suppliers_success() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        let expected_suppliers = vec![
            create_test_supplier("sup1", "Supplier Alpha"),
            create_test_supplier("sup2", "Supplier Beta"),
        ];
        let expected_suppliers_cl = expected_suppliers.clone();

        mock_repo.expect_find_all()
            .times(1)
            .returning(move |_conn| {
                let suppliers = expected_suppliers_cl.clone();
                Box::pin(async move { Ok(suppliers) })
            });

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;

        let result = service.get_all_suppliers(pool).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_suppliers);
    }
    
    #[tokio::test]
    async fn test_get_all_suppliers_empty() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        mock_repo.expect_find_all()
            .times(1)
            .returning(move |_conn| Box::pin(async move { Ok(Vec::new()) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;

        let result = service.get_all_suppliers(pool).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_update_supplier_success() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        let id = "sup-to-update".to_string();
        let name = "Updated Name".to_string();
        let jenis_barang = "Updated Jenis".to_string();
        let jumlah_barang = 20;
        let resi = "UPDATEDRESI".to_string();

        let id_cl = id.clone();
        let name_cl = name.clone();
        let jumlah_barang_cl = jumlah_barang;

        mock_repo.expect_update()
            .withf(move |s: &Supplier, _conn: &PoolConnection<Any>| {
                s.id == id_cl && s.name == name_cl && s.jumlah_barang == jumlah_barang_cl
            })
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;
        let result = service.update_supplier(pool, id.clone(), name, jenis_barang, jumlah_barang, resi).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_supplier_not_found() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        mock_repo.expect_update()
            .times(1)
            .returning(|_, _| Box::pin(async { Err(SqlxError::RowNotFound) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;
        let result = service.update_supplier(pool, "non-existent".to_string(), "N".to_string(), "J".to_string(), 1, "R".to_string()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Service: Supplier not found for update.");
    }

    #[tokio::test]
    async fn test_delete_supplier_success() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();
        let supplier_id = "sup-to-delete";

        mock_repo.expect_delete()
            .with(eq(supplier_id), always())
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;
        let result = service.delete_supplier(pool, supplier_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_supplier_not_found() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        mock_repo.expect_delete()
            .times(1)
            .returning(|_, _| Box::pin(async { Err(SqlxError::RowNotFound) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;
        let result = service.delete_supplier(pool, "non-existent").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Service: Supplier not found for delete.");
    }

     #[tokio::test]
    async fn test_get_supplier_acquire_connection_fails() {
        let mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        let service = SupplierServiceImpl::new(
            Arc::new(mock_repo),
            Arc::new(mock_transaction_repo),
            Arc::new(mock_notifier),
        );

        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create dummy pool for close test");
        pool.close().await;

        let result = service.get_supplier(pool, "any_id").await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.starts_with("Service: Failed to acquire DB connection:"));
        assert!(err_msg.contains("closed pool"));
    }

    #[tokio::test]
    async fn test_get_supplier_repository_generic_error() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();
        let supplier_id = "sup-repo-err";

        mock_repo.expect_find_by_id()
            .with(eq(supplier_id), always())
            .times(1)
            .returning(|_, _| Box::pin(async { Err(SqlxError::PoolTimedOut) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;

        let result = service.get_supplier(pool, supplier_id).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.starts_with("Service: Repository error:"));
        assert!(err_msg.contains("pool timed out")); 
    }

    #[tokio::test]
    async fn test_get_all_suppliers_acquire_connection_fails() {
        let mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        let service = SupplierServiceImpl::new(
            Arc::new(mock_repo),
            Arc::new(mock_transaction_repo),
            Arc::new(mock_notifier),
        );
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("Pool creation failed");
        pool.close().await;

        let result = service.get_all_suppliers(pool).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.starts_with("Service: Failed to acquire DB connection:"));
        assert!(err_msg.contains("closed pool"));
    }

    #[tokio::test]
    async fn test_get_all_suppliers_repository_generic_error() {
        let mut mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        mock_repo.expect_find_all()
            .times(1)
            .returning(|_conn| Box::pin(async { Err(SqlxError::PoolTimedOut) }));

        let service = SupplierServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_transaction_repo), Arc::new(mock_notifier));
        let pool = create_dummy_pool().await;

        let result = service.get_all_suppliers(pool).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.starts_with("Service: Repository error:"));
        assert!(err_msg.contains("pool timed out"));
    }

    #[tokio::test]
    async fn test_get_all_supplier_transactions_acquire_connection_fails() {
        let mock_repo = MockSupplierRepository::new();
        let mock_notifier = MockSupplierNotifier::new();
        let mock_transaction_repo = MockSupplierTransactionRepository::new();

        let service = SupplierServiceImpl::new(
            Arc::new(mock_repo),
            Arc::new(mock_transaction_repo),
            Arc::new(mock_notifier),
        );
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("Pool creation failed");
        pool.close().await;

        let result = service.get_all_supplier_transactions(pool).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.starts_with("Service: Failed to acquire DB connection:"));
        assert!(err_msg.contains("closed pool"));
    }
}