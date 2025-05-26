use chrono::Utc;
use uuid::Uuid;

use crate::manajemen_supplier::model::{supplier::Supplier, supplier_transaction::SupplierTransaction};

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
            tanggal_transaksi: Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::manajemen_supplier::patterns::factory::SupplierTransactionFactory;
    use crate::manajemen_supplier::model::supplier::Supplier;

    #[test]
    fn test_create_transaction_from_supplier() {
        let supplier_id = format!("SUP-{}", Uuid::new_v4());
        let now = Utc::now();

        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: now.to_rfc3339(),
        };

        let transaction = SupplierTransactionFactory::create_from_supplier(&supplier);

        assert_eq!(transaction.supplier_id, supplier.id);
        assert_eq!(transaction.supplier_name, supplier.name);
        assert_eq!(transaction.jenis_barang, supplier.jenis_barang);
        assert_eq!(transaction.jumlah_barang, supplier.jumlah_barang);
        assert_eq!(transaction.pengiriman_info, supplier.resi);
    }
}