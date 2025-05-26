use serde::{Deserialize, Serialize};

use super::supplier::Supplier;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupplierTransaction {
    pub id: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub pengiriman_info: String,
    pub tanggal_transaksi: String,
}

impl SupplierTransaction {
    pub fn from_supplier(id: String, supplier: &Supplier) -> Self {
        Self {
            id,
            supplier_id: supplier.id.clone(),
            supplier_name: supplier.name.clone(),
            jenis_barang: supplier.jenis_barang.clone(),
            jumlah_barang: supplier.jumlah_barang,
            pengiriman_info: supplier.resi.clone(),
            tanggal_transaksi: supplier.updated_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_supplier_to_transaction() {
        let now = Utc::now();

        let supplier = Supplier {
            id: "SUP-001".to_string(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 100,
            resi: "2306206282".to_string(),
            updated_at: now.to_rfc3339(),
        };

        let transaksi = SupplierTransaction::from_supplier("STRX-001".to_string(), &supplier);

        assert_eq!(transaksi.supplier_id, "SUP-001");
        assert_eq!(transaksi.supplier_name, "PT. Ayam");
        assert_eq!(transaksi.jenis_barang, "Ayam");
        assert_eq!(transaksi.jumlah_barang, 100);
        assert_eq!(transaksi.pengiriman_info, "2306206282");
        assert_eq!(transaksi.tanggal_transaksi, now.to_rfc3339());
    }
}
