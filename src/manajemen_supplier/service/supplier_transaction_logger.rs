use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Any, Pool};
use log::{error, info};

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::patterns::factory::SupplierTransactionFactory;
use crate::manajemen_supplier::{repository::supplier_transaction_repository::SupplierTransactionRepository, service::supplier_observer::SupplierObserver};

#[derive(Clone)]
pub struct SupplierTransactionLogger {
    trx_repo: Arc<dyn SupplierTransactionRepository + Send + Sync>,
    db_pool: Pool<Any>,
}

impl SupplierTransactionLogger {
    pub fn new(
        trx_repo: Arc<dyn SupplierTransactionRepository + Send + Sync>,
        db_pool: Pool<Any>,
    ) -> Self {
        Self { trx_repo, db_pool }
    }
}

#[async_trait]
impl SupplierObserver for SupplierTransactionLogger {
    async fn on_supplier_saved(&self, supplier: &Supplier) {
        let transaction_to_save = SupplierTransactionFactory::create_from_supplier(supplier);
        
        match self.db_pool.acquire().await {
            Ok(conn) => { 
                if let Err(err) = self.trx_repo.save(transaction_to_save, conn).await { 
                    error!("[ERROR] Failed to log supplier transaction for supplier ID {}: {}", supplier.id, err);
                } else {
                    info!("[INFO] Successfully logged transaction for supplier ID {}", supplier.id);
                }
            }
            Err(err) => {
                error!("[ERROR] Failed to acquire DB connection for logging transaction for supplier ID {}: {}", supplier.id, err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mockall::predicate::*;
    use sqlx::any::AnyPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
    use crate::manajemen_supplier::repository::supplier_transaction_repository::{
        MockSupplierTransactionRepository, 
    };

    async fn create_dummy_pool_for_logger_tests() -> Pool<Any> {
        sqlx::any::install_default_drivers();
        AnyPoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:file:memdb_logger_test_{}?mode=memory&cache=shared", Uuid::new_v4()))
            .await
            .expect("Failed to create dummy pool for logger tests")
    }

    fn sample_supplier_for_logger_tests() -> Supplier {
        Supplier {
            id: "SUP-LOG-TEST-001".to_string(),
            name: "Logging Test Inc.".to_string(),
            jenis_barang: "Data".to_string(),
            jumlah_barang: 42,
            resi: "LOGRESI001".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_on_supplier_saved_logs_transaction_successfully() {
        let mut mock_trx_repo = MockSupplierTransactionRepository::new();
        let dummy_pool = create_dummy_pool_for_logger_tests().await;
        let supplier_arg = sample_supplier_for_logger_tests();
        
        let expected_supplier_id_in_trx = supplier_arg.id.clone();
        let expected_supplier_name_in_trx = supplier_arg.name.clone();

        mock_trx_repo.expect_save()
            .withf(move |trx: &SupplierTransaction, _conn| {
                trx.supplier_id == expected_supplier_id_in_trx && 
                trx.supplier_name == expected_supplier_name_in_trx
            })
            .times(1)
            .returning(|trx_input, _conn| { 
                Ok(SupplierTransaction { 
                    id: "mocked-trx-id-123".to_string(), 
                    supplier_id: trx_input.supplier_id.clone(), 
                    supplier_name: trx_input.supplier_name.clone(), 
                    jenis_barang: trx_input.jenis_barang.clone(),
                    jumlah_barang: trx_input.jumlah_barang,
                    pengiriman_info: trx_input.pengiriman_info.clone(), 
                    tanggal_transaksi: Utc::now().to_rfc3339()
                })
            });

        let logger = SupplierTransactionLogger::new(Arc::new(mock_trx_repo), dummy_pool);
        logger.on_supplier_saved(&supplier_arg).await;
    }

    #[tokio::test]
    async fn test_on_supplier_saved_handles_repository_save_error() {
        let mut mock_trx_repo = MockSupplierTransactionRepository::new();
        let dummy_pool = create_dummy_pool_for_logger_tests().await;
        let supplier_arg = sample_supplier_for_logger_tests();

        mock_trx_repo.expect_save()
            .with(always(), mockall::predicate::always()) 
            .times(1)
            .returning(|_trx, _conn| Err(sqlx::Error::Protocol("Simulated DB save error".to_string())));

        let logger = SupplierTransactionLogger::new(Arc::new(mock_trx_repo), dummy_pool);
        logger.on_supplier_saved(&supplier_arg).await;
    }
}
