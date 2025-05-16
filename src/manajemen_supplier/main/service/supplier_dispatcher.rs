use std::sync::{Arc, Mutex};

use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::main::service::supplier_observer::SupplierObserver;


impl SupplierNotifier for SupplierDispatcher {
    fn notify_supplier_saved(&self, supplier: &Supplier) {
        self.notify_supplier_saved(supplier);
    }
}


#[derive(Clone)]
pub struct SupplierDispatcher {
    observers: Arc<Mutex<Vec<Arc<dyn SupplierObserver>>>>,
}

impl SupplierDispatcher {
    pub fn new() -> Self {
        Self {
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