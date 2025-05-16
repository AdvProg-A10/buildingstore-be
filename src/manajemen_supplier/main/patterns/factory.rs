use chrono::Utc;
use uuid::Uuid;

use crate::manajemen_supplier::main::model::{supplier::Supplier, supplier_transaction::SupplierTransaction};

pub struct SupplierTransactionFactory;

impl SupplierTransactionFactory {
    pub fn create_from_supplier(supplier: &Supplier) -> SupplierTransaction {
        SupplierTransaction {
            id: format!("TRX-{}", Uuid::new_v4()),
            supplier_id: supplier.id.clone(),
            supplier_name: supplier.name.clone(),
            jenis_barang: supplier.jenis_barang.clone(),
            jumlah_barang: supplier.jumlah_barang,
            pengiriman_info: supplier.resi.clone(),
            tanggal_transaksi: Utc::now(),
        }
    }
}