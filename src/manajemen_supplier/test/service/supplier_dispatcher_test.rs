#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use crate::manajemen_supplier::main::model::supplier::Supplier;
    use crate::manajemen_supplier::main::service::supplier_dispatcher::SupplierDispatcher;
    use crate::manajemen_supplier::main::service::supplier_notifier::SupplierNotifier;
    use crate::manajemen_supplier::main::service::supplier_observer::SupplierObserver;

    struct MockObserver {
        pub called: Arc<Mutex<bool>>,
        pub last_supplier_name: Arc<Mutex<Option<String>>>,
    }

    impl SupplierObserver for MockObserver {
        fn on_supplier_saved(&self, supplier: &Supplier) {
            let mut called = self.called.lock().unwrap();
            let mut name = self.last_supplier_name.lock().unwrap();
            *called = true;
            *name = Some(supplier.name.clone());
        }
    }

    fn sample_supplier() -> Supplier {
        Supplier {
            id: "SUP-001".to_string(),
            name: "Test Supplier".to_string(),
            jenis_barang: "Elektronik".to_string(),
            jumlah_barang: 100,
            resi: "RESI123".to_string(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_register_and_notify_observer() {
        let dispatcher = SupplierDispatcher::new();

        let called_flag = Arc::new(Mutex::new(false));
        let last_name = Arc::new(Mutex::new(None));

        let observer = Arc::new(MockObserver {
            called: called_flag.clone(),
            last_supplier_name: last_name.clone(),
        });

        dispatcher.register(observer.clone());
        let supplier = sample_supplier();
        dispatcher.notify_supplier_saved(&supplier);

        assert_eq!(*called_flag.lock().unwrap(), true);
        assert_eq!(last_name.lock().unwrap().as_deref(), Some("Test Supplier"));
    }

    #[test]
    fn test_supplier_notifier_trait_works() {
        let dispatcher = SupplierDispatcher::new();
        let notifier: Arc<dyn SupplierNotifier> = Arc::new(dispatcher.clone());

        let called_flag = Arc::new(Mutex::new(false));
        let last_name = Arc::new(Mutex::new(None));

        let observer = Arc::new(MockObserver {
            called: called_flag.clone(),
            last_supplier_name: last_name.clone(),
        });

        dispatcher.register(observer);
        let supplier = sample_supplier();

        notifier.notify_supplier_saved(&supplier);

        assert_eq!(*called_flag.lock().unwrap(), true);
        assert_eq!(last_name.lock().unwrap().as_deref(), Some("Test Supplier"));
    }
}
