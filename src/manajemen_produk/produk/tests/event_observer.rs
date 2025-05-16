use crate::manajemen_produk::produk::events::ProdukObserver;
use crate::manajemen_produk::produk::model::Produk;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct MockObserver {
    notified: Arc<Mutex<u32>>,
}

impl MockObserver {
    fn new() -> Self {
        Self {
            notified: Arc::new(Mutex::new(0)),
        }
    }

    fn count(&self) -> u32 {
        *self.notified.lock().unwrap()
    }
}

impl ProdukObserver for MockObserver {
    fn on_stock_changed(&self, _produk: &Produk, _old_stock: u32) {
        let mut count = self.notified.lock().unwrap();
        *count += 1;
    }
}

#[test]
fn test_stock_updated_event() {
    let mut produk = Produk::new(
        "Laptop".to_string(),
        "Elektronik".to_string(),
        1000.0,
        10,
        None,
    );

    let observer = Arc::new(MockObserver::new());

    produk.add_observer(observer.clone());
    produk.set_stok(5); // Should trigger observer

    assert_eq!(observer.count(), 1);
}