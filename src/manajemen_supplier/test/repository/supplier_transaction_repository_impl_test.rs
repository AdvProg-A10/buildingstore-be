use chrono::Utc;
use uuid::Uuid;

use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::model::supplier_transaction::SupplierTransaction;
use crate::manajemen_supplier::main::repository::supplier_transaction_repository::SupplierTransactionRepository;
use crate::manajemen_supplier::main::repository::supplier_transaction_repository_impl::SupplierTransactionRepositoryImpl;

#[test]
fn save_supplier_transaction() {
    let repository = SupplierTransactionRepositoryImpl::new();
    let supplier_id = format!("SUP-{}", Uuid::new_v4());

    let supplier = Supplier {
        id: supplier_id.clone(),
        name: "PT. Ayam".to_string(),
        jenis_barang: "ayam".to_string(),
        jumlah_barang: 1000,
        resi: "2306206282".to_string(),
        updated_at: Utc::now(),
    };

    let transaksi = SupplierTransaction {
        id: format!("TRX-{}", Uuid::new_v4()),
        supplier_id: supplier.id.clone(),
        supplier_name: supplier.name.clone(),
        jenis_barang: supplier.jenis_barang.clone(),
        jumlah_barang: supplier.jumlah_barang,
        pengiriman_info: supplier.resi.clone(),
        tanggal_transaksi: Utc::now(),
    };

    let result = repository.save(transaksi.clone());
    assert!(result.is_ok());
    let saved = result.unwrap();
    assert_eq!(saved.supplier_id, supplier.id);
    assert_eq!(saved.supplier_name, supplier.name);
}

#[test]
fn test_find_supplier_transaction_by_id() {
    let repository = SupplierTransactionRepositoryImpl::new();
    let supplier_id = format!("SUP-{}", Uuid::new_v4());

    let supplier = Supplier {
        id: supplier_id.clone(),
        name: "PT. Ayam".to_string(),
        jenis_barang: "ayam".to_string(),
        jumlah_barang: 1000,
        resi: "2306206282".to_string(),
        updated_at: Utc::now(),
    };

    let transaksi = SupplierTransaction {
        id: format!("TRX-{}", Uuid::new_v4()),
        supplier_id: supplier.id.clone(),
        supplier_name: supplier.name.clone(),
        jenis_barang: supplier.jenis_barang.clone(),
        jumlah_barang: supplier.jumlah_barang,
        pengiriman_info: supplier.resi.clone(),
        tanggal_transaksi: Utc::now(),
    };

    repository.save(transaksi.clone()).unwrap();
    let found = repository.find_by_id(&transaksi.id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, transaksi.id);
}

#[test]
fn test_find_supplier_transaction_by_supplier_id() {
    let repository = SupplierTransactionRepositoryImpl::new();
    let supplier_id = format!("SUP-{}", Uuid::new_v4());

    let supplier = Supplier {
        id: supplier_id.clone(),
        name: "PT. Ayam".to_string(),
        jenis_barang: "ayam".to_string(),
        jumlah_barang: 1000,
        resi: "2306206282".to_string(),
        updated_at: Utc::now(),
    };

    let transaksi = SupplierTransaction {
        id: format!("TRX-{}", Uuid::new_v4()),
        supplier_id: supplier.id.clone(),
        supplier_name: supplier.name.clone(),
        jenis_barang: supplier.jenis_barang.clone(),
        jumlah_barang: supplier.jumlah_barang,
        pengiriman_info: supplier.resi.clone(),
        tanggal_transaksi: Utc::now(),
    };

    repository.save(transaksi.clone()).unwrap();
    let results = repository.find_by_supplier_id(&supplier.id);
    assert!(!results.is_empty());
    assert_eq!(results[0].supplier_id, supplier.id);
}