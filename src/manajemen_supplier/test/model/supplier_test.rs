#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use chrono::Utc;
    use crate::manajemen_supplier::main::model::supplier::Supplier;

    #[test]
    fn test_create_supplier() {
        let supplier_id = format!("SUP-{}", Uuid::new_v4());

        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        };

        assert_eq!(supplier.name, "PT. Ayam");
        assert_eq!(supplier.jenis_barang, "ayam");
        assert_eq!(supplier.jumlah_barang, 1000);
        assert_eq!(supplier.resi, "2306206282");
    }
}
