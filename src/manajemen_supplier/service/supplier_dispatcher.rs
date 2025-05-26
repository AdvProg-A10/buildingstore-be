use std::sync::{Arc, Mutex};
use futures::future::join_all;
use async_trait::async_trait;
use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_observer::SupplierObserver;

#[derive(Clone)]
pub struct SupplierDispatcher {
    observers: Arc<Mutex<Vec<Arc<dyn SupplierObserver + Send + Sync>>>>,
}

impl SupplierDispatcher {
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&self, observer: Arc<dyn SupplierObserver + Send + Sync>) {
        self.observers.lock().unwrap().push(observer);
    }

    async fn notify_observers_on_save(&self, supplier: &Supplier) {
        let observers_to_notify = self.observers.lock().unwrap().clone();
        
        if observers_to_notify.is_empty() {
            return;
        }

        let futures = observers_to_notify.into_iter()
            .map(|observer_arc| {
                async move {
                    observer_arc.on_supplier_saved(supplier).await;
                }
            });
        join_all(futures).await;
    }
}

#[async_trait]
impl SupplierNotifier for SupplierDispatcher {
    async fn notify_supplier_saved(&self, supplier: &Supplier) {
        self.notify_observers_on_save(supplier).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicBool, Ordering};
    use chrono::Utc;
    use async_trait::async_trait;
    use crate::manajemen_supplier::model::supplier::Supplier;

    struct MockObserver {
        called: Arc<AtomicBool>,
        last_supplier_name: Arc<Mutex<Option<String>>>, 
    }

    impl MockObserver {
        fn new(called_flag: Arc<AtomicBool>, name_store: Arc<Mutex<Option<String>>>) -> Self {
            Self { called: called_flag, last_supplier_name: name_store }
        }
    }

    #[async_trait]
    impl SupplierObserver for MockObserver {
        async fn on_supplier_saved(&self, supplier: &Supplier) {
            self.called.store(true, Ordering::Relaxed);
            let mut name_lock = self.last_supplier_name.lock().unwrap();
            *name_lock = Some(supplier.name.clone());
        }
    }

    fn sample_supplier_for_dispatcher_tests() -> Supplier { 
        Supplier {
            id: "DISP-SUP-001".to_string(),
            name: "Dispatcher Test Supplier".to_string(),
            jenis_barang: "Elektronik".to_string(),
            jumlah_barang: 100,
            resi: "DISPRESI123".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_register_and_notify_single_observer() {
        let dispatcher = SupplierDispatcher::new();

        let called_flag = Arc::new(AtomicBool::new(false));
        let last_name_store = Arc::new(Mutex::new(None::<String>));
        let observer = Arc::new(MockObserver::new(called_flag.clone(), last_name_store.clone()));

        dispatcher.register(observer);
        let supplier = sample_supplier_for_dispatcher_tests();
        dispatcher.notify_supplier_saved(&supplier).await;

        assert_eq!(called_flag.load(Ordering::Relaxed), true, "Observer was not called");
        assert_eq!(
            last_name_store.lock().unwrap().as_deref(),
            Some("Dispatcher Test Supplier"),
            "Supplier name was not correctly recorded"
        );
    }

    #[tokio::test]
    async fn test_notify_multiple_observers() {
        let dispatcher = SupplierDispatcher::new();

        let called_flag1 = Arc::new(AtomicBool::new(false));
        let last_name_store1 = Arc::new(Mutex::new(None::<String>));
        let observer1 = Arc::new(MockObserver::new(called_flag1.clone(), last_name_store1.clone()));

        let called_flag2 = Arc::new(AtomicBool::new(false));
        let last_name_store2 = Arc::new(Mutex::new(None::<String>));
        let observer2 = Arc::new(MockObserver::new(called_flag2.clone(), last_name_store2.clone()));

        dispatcher.register(observer1);
        dispatcher.register(observer2);

        let supplier = sample_supplier_for_dispatcher_tests();
        dispatcher.notify_supplier_saved(&supplier).await;

        assert_eq!(called_flag1.load(Ordering::Relaxed), true, "Observer 1 was not called");
        assert_eq!(last_name_store1.lock().unwrap().as_deref(), Some("Dispatcher Test Supplier"));

        assert_eq!(called_flag2.load(Ordering::Relaxed), true, "Observer 2 was not called");
        assert_eq!(last_name_store2.lock().unwrap().as_deref(), Some("Dispatcher Test Supplier"));
    }
    
    #[tokio::test]
    async fn test_notify_no_observers() {
        let dispatcher = SupplierDispatcher::new(); 
        let supplier = sample_supplier_for_dispatcher_tests();
        
        dispatcher.notify_supplier_saved(&supplier).await;
    }

    #[tokio::test]
    async fn test_supplier_notifier_trait_works_via_dispatcher() {
        let dispatcher = SupplierDispatcher::new();
        let notifier: Arc<dyn SupplierNotifier> = Arc::new(dispatcher.clone()); 

        let called_flag = Arc::new(AtomicBool::new(false));
        let last_name_store = Arc::new(Mutex::new(None::<String>));
        let observer = Arc::new(MockObserver::new(called_flag.clone(), last_name_store.clone()));

        dispatcher.register(observer); 
        let supplier = sample_supplier_for_dispatcher_tests();
        notifier.notify_supplier_saved(&supplier).await;

        assert_eq!(called_flag.load(Ordering::Relaxed), true, "Observer was not called via trait");
        assert_eq!(
            last_name_store.lock().unwrap().as_deref(),
            Some("Dispatcher Test Supplier"),
            "Supplier name not recorded via trait"
        );
    }
}