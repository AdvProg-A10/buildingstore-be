use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::manajemen_supplier::main::model::supplier_transaction::SupplierTransaction;
use crate::manajemen_supplier::main::repository::supplier_transaction_repository::SupplierTransactionRepository;

#[derive(Clone)]
pub struct SupplierTransactionRepositoryImpl {
    pub storage: Arc<Mutex<HashMap<String, SupplierTransaction>>>,
}

impl SupplierTransactionRepositoryImpl {
    pub fn new() -> Self {
        SupplierTransactionRepositoryImpl {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SupplierTransactionRepository for SupplierTransactionRepositoryImpl {
    fn save(&self, transaction: SupplierTransaction) -> Result<SupplierTransaction, String> {
        match self.storage.lock() {
            Ok(mut store) => {
                store.insert(transaction.id.clone(), transaction.clone());
                Ok(transaction)
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    fn find_by_id(&self, id: &str) -> Option<SupplierTransaction> {
        match self.storage.lock() {
            Ok(store) => store.get(id).cloned(),
            Err(_) => None,
        }
    }

    fn find_by_supplier_id(&self, supplier_id: &str) -> Vec<SupplierTransaction> {
        match self.storage.lock() {
            Ok(store) => store
                .values()
                .filter(|trx| trx.supplier_id == supplier_id)
                .cloned()
                .collect(),
            Err(_) => vec![],
        }
    }
}
