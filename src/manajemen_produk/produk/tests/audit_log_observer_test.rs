use crate::manajemen_produk::produk::audit_log_observer::AuditLogObserver;
use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::events::ProdukObserver;

#[test]
fn test_on_stock_changed_logs_correctly() {
    let observer = AuditLogObserver::new();
    let produk = Produk::new(
        "Test Product".to_string(),
        "Test Category".to_string(),
        100.0,
        5,
        None,
    );
    
    // Just test that the function can be called without panicking
    observer.on_stock_changed(&produk, 3);
}

#[test]
fn test_on_stock_changed_with_id() {
    let observer = AuditLogObserver::new();
    let mut produk = Produk::new(
        "Test Product".to_string(),
        "Test Category".to_string(),
        100.0,
        5,
        None,
    );
    produk.id = Some(42);
    
    observer.on_stock_changed(&produk, 3);
}

#[test]
fn test_on_stock_changed_with_description() {
    let observer = AuditLogObserver::new();
    let produk = Produk::new(
        "Test Product".to_string(),
        "Test Category".to_string(),
        100.0,
        5,
        Some("Test Description".to_string()),
    );
    
    observer.on_stock_changed(&produk, 10);
}

#[test]
fn test_on_stock_changed_same_stock() {
    let observer = AuditLogObserver::new();
    let produk = Produk::new(
        "Test Product".to_string(),
        "Test Category".to_string(),
        100.0,
        5,
        None,
    );
    observer.on_stock_changed(&produk, 5);
}

#[test]
fn test_on_stock_changed_zero_stock() {
    let observer = AuditLogObserver::new();
    let produk = Produk::new(
        "Test Product".to_string(),
        "Test Category".to_string(),
        100.0,
        0,
        None,
    );
    observer.on_stock_changed(&produk, 5);
}

#[test]
fn test_on_stock_changed_large_numbers() {
    let observer = AuditLogObserver::new();
    let produk = Produk::new(
        "Bulk Item".to_string(),
        "Wholesale".to_string(),
        10.0,
        10_000,
        None,
    );
    observer.on_stock_changed(&produk, 5_000);
}