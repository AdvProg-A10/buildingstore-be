#[cfg(test)]
mod tests {
    use crate::manajemen_supplier::main::model::{supplier::Supplier, supplier_transaction::SupplierTransaction};
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
            updated_at: now,
        };

        let transaksi = SupplierTransaction {
            id: "STRX-001".to_string(),
            supplier_id: supplier.id.clone(),
            supplier_name: supplier.name.clone(),
            jenis_barang: supplier.jenis_barang.clone(),
            jumlah_barang: supplier.jumlah_barang,
            pengiriman_info: supplier.resi.clone(),
            tanggal_transaksi: supplier.updated_at,
        };

        assert_eq!(transaksi.supplier_id, "SUP-001");
        assert_eq!(transaksi.supplier_name, "PT. Ayam");
        assert_eq!(transaksi.jenis_barang, "Ayam");
        assert_eq!(transaksi.jumlah_barang, 100);
        assert_eq!(transaksi.pengiriman_info, "2306206282");
        assert_eq!(transaksi.tanggal_transaksi, now);
    }
}
