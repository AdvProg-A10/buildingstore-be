#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::manajemen_supplier::main::model::supplier::Supplier;
    use crate::manajemen_supplier::main::repository::supplier_transaction_repository::SupplierTransactionRepository;
    use crate::manajemen_supplier::main::repository::supplier_transaction_repository_impl::SupplierTransactionRepositoryImpl;
    use crate::manajemen_supplier::main::patterns::observer::{SupplierEventDispatcher, SupplierTransactionLogger};

    #[test]
    fn test_supplier_observer_logs_transaction_on_save() {
        // Arrange
        let trx_repo = Arc::new(SupplierTransactionRepositoryImpl::new());
        let logger = Arc::new(SupplierTransactionLogger {
            trx_repo: trx_repo.clone(),
        });

        let dispatcher = SupplierEventDispatcher::new();
        dispatcher.register(logger);

        let supplier_id = format!("SUP-{}", Uuid::new_v4());
        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 100,
            resi: "123ABC".to_string(),
            updated_at: Utc::now(),
        };

        dispatcher.notify_supplier_saved(&supplier);

        let transactions = trx_repo.find_by_supplier_id(&supplier_id);
        assert_eq!(transactions.len(), 1);

        let trx = &transactions[0];
        assert_eq!(trx.supplier_id, supplier.id);
        assert_eq!(trx.supplier_name, supplier.name);
        assert_eq!(trx.jenis_barang, supplier.jenis_barang);
        assert_eq!(trx.jumlah_barang, supplier.jumlah_barang);
        assert_eq!(trx.pengiriman_info, supplier.resi);
    }
}
