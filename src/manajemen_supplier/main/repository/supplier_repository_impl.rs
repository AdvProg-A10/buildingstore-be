use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::repository::supplier_repository::SupplierRepository;

#[derive(Clone, Default)]
pub struct SupplierRepositoryImpl {
    store: Arc<Mutex<HashMap<String, Supplier>>>,
}

impl SupplierRepositoryImpl {
    pub fn new() -> Self {
        SupplierRepositoryImpl {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SupplierRepository for SupplierRepositoryImpl {
    fn save(&self, supplier: Supplier) -> Result<Supplier, String> {
        let mut store = self.store.lock().unwrap();
        store.insert(supplier.id.clone(), supplier.clone());
        Ok(supplier)
    }

    fn find_by_id(&self, id: &str) -> Option<Supplier> {
        let store = self.store.lock().unwrap();
        store.get(id).cloned()
    }

    fn update(&self, supplier: Supplier) -> Result<(), String> {
        let mut store = self.store.lock().unwrap();
        if store.contains_key(&supplier.id) {
            store.insert(supplier.id.clone(), supplier);
            Ok(())
        } else {
            Err("Supplier not found".to_string())
        }
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let mut store = self.store.lock().unwrap();
        if store.remove(id).is_some() {
            Ok(())
        } else {
            Err("Supplier not found".to_string())
        }
    }
}
