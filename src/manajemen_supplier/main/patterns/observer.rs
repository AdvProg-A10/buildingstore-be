use std::sync::{Arc, Mutex};

use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::repository::supplier_transaction_repository::SupplierTransactionRepository;
use crate::manajemen_supplier::main::patterns::factory::SupplierTransactionFactory;
use crate::manajemen_supplier::main::repository::supplier_transaction_repository_impl::SupplierTransactionRepositoryImpl;

pub trait SupplierObserver: Send + Sync {
    fn on_supplier_saved(&self, supplier: &Supplier);
}

#[derive(Clone)]
pub struct SupplierEventDispatcher {
    observers: Arc<Mutex<Vec<Arc<dyn SupplierObserver>>>>,
}

impl SupplierEventDispatcher {
    pub fn new() -> Self {
        SupplierEventDispatcher {
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&self, observer: Arc<dyn SupplierObserver>) {
        self.observers.lock().unwrap().push(observer);
    }

    pub fn notify_supplier_saved(&self, supplier: &Supplier) {
        let observers = self.observers.lock().unwrap();
        for obs in observers.iter() {
            obs.on_supplier_saved(supplier);
        }
    }
}

#[derive(Clone)]
pub struct SupplierTransactionLogger {
    pub trx_repo: Arc<SupplierTransactionRepositoryImpl>,
}

impl SupplierObserver for SupplierTransactionLogger {
    fn on_supplier_saved(&self, supplier: &Supplier) {
        let trx = SupplierTransactionFactory::create_from_supplier(supplier);
        let _ = self.trx_repo.save(trx);
    }
}
