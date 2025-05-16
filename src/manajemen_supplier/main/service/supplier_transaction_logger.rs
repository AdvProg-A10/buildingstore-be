use std::sync::Arc;

use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::patterns::factory::SupplierTransactionFactory;
use crate::manajemen_supplier::main::patterns::observer::SupplierObserver;
use crate::manajemen_supplier::main::repository::supplier_transaction_repository::SupplierTransactionRepository;

#[derive(Clone)]
pub struct SupplierTransactionLogger {
    pub trx_repo: Arc<dyn SupplierTransactionRepository + Send + Sync>,
}

impl SupplierTransactionLogger {
    pub fn new(trx_repo: Arc<dyn SupplierTransactionRepository + Send + Sync>) -> Self {
        Self { trx_repo }
    }
}

impl SupplierObserver for SupplierTransactionLogger {
    fn on_supplier_saved(&self, supplier: &Supplier) {
        let trx = SupplierTransactionFactory::create_from_supplier(supplier);
        if let Err(err) = self.trx_repo.save(trx) {
            eprintln!("Failed to log transaction: {}", err);
        }
    }
}
