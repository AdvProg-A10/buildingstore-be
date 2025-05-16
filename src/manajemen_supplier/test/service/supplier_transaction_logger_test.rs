#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;

    use crate::manajemen_supplier::main::model::supplier::Supplier;
    use crate::manajemen_supplier::main::model::supplier_transaction::SupplierTransaction;
    use crate::manajemen_supplier::main::repository::supplier_transaction_repository::SupplierTransactionRepository;
    use crate::manajemen_supplier::main::patterns::observer::SupplierObserver;
    use crate::manajemen_supplier::main::service::supplier_transaction_logger::SupplierTransactionLogger;

    struct MockSupplierTransactionRepository {
        pub saved_transaction: Arc<Mutex<Option<SupplierTransaction>>>,
    }

    impl SupplierTransactionRepository for MockSupplierTransactionRepository {
        fn save(&self, trx: SupplierTransaction) -> Result<SupplierTransaction, std::string::String> {
            let mut lock = self.saved_transaction.lock().unwrap();
            *lock = Some(trx.clone());
            Ok(trx)
        }

        fn find_by_id(&self, _id: &str) -> Option<SupplierTransaction> {
            None
        }
    
        fn find_by_supplier_id(&self, _supplier_id: &str) -> Vec<SupplierTransaction> {
            vec![]
        }
    }

    fn sample_supplier() -> Supplier {
        Supplier {
            id: "SUP-123".to_string(),
            name: "PT. Pt".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 10,
            resi: "RESI123".to_string(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_on_supplier_saved_logs_transaction() {
        let saved_trx = Arc::new(Mutex::new(None));

        let mock_repo = Arc::new(MockSupplierTransactionRepository {
            saved_transaction: saved_trx.clone(),
        });

        let logger = SupplierTransactionLogger::new(mock_repo);
        let supplier = sample_supplier();
        logger.on_supplier_saved(&supplier);

        let logged = saved_trx.lock().unwrap();
        assert!(logged.is_some());
        let trx = logged.as_ref().unwrap();

        assert_eq!(trx.supplier_id, "SUP-123");
        assert_eq!(trx.supplier_name, "PT. Pt");
        assert_eq!(trx.jenis_barang, "Ayam");
        assert_eq!(trx.jumlah_barang, 10);
        assert_eq!(trx.pengiriman_info, "RESI123");
    }
}
