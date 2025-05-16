#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use chrono::Utc;

    use crate::manajemen_supplier::main::model::supplier::Supplier;
    use crate::manajemen_supplier::main::repository::supplier_repository_impl::SupplierRepositoryImpl;
    use crate::manajemen_supplier::main::repository::supplier_repository::SupplierRepository;
    use crate::manajemen_supplier::main::service::supplier_service_impl::SupplierServiceImpl;
    use crate::manajemen_supplier::main::service::supplier_service::SupplierService;
    use crate::manajemen_supplier::main::service::supplier_notifier::SupplierNotifier;

    struct MockNotifier;

    impl SupplierNotifier for MockNotifier {
        fn notify_supplier_saved(&self, _supplier: &Supplier) {
            // Do nothing for tests
        }
    }

    fn create_sample_supplier() -> Supplier {
        Supplier {
            id: "SUP-123".to_string(),
            name: "PT. pt".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 10,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_save_supplier_success() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let supplier = create_sample_supplier();
        let result = service.save_supplier(supplier.clone());

        assert!(result.is_ok());
        let stored = repo.find_by_id("SUP-123").unwrap();
        assert_eq!(stored.name, supplier.name);
    }

    #[test]
    fn test_update_supplier_success() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let mut supplier = create_sample_supplier();
        service.save_supplier(supplier.clone()).unwrap();

        supplier.name = "Updated Supplier".to_string();
        let update_result = service.update_supplier(supplier.clone());

        assert!(update_result.is_ok());
        let updated = repo.find_by_id("SUP-123").unwrap();
        assert_eq!(updated.name, "Updated Supplier");
    }

    #[test]
    fn test_update_supplier_failure() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let supplier = create_sample_supplier();
        let result = service.update_supplier(supplier);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Supplier not found".to_string());
    }

    #[test]
    fn test_delete_supplier_success() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let supplier = create_sample_supplier();
        service.save_supplier(supplier.clone()).unwrap();

        let delete_result = service.delete_supplier(&supplier.id);
        assert!(delete_result.is_ok());

        let found = repo.find_by_id(&supplier.id);
        assert!(found.is_none());
    }

    #[test]
    fn test_delete_supplier_failure() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let result = service.delete_supplier("UNKNOWN-ID");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Supplier not found".to_string());
    }

    #[test]
    fn test_get_supplier_success() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let supplier = create_sample_supplier();
        service.save_supplier(supplier.clone()).unwrap();

        let result = service.get_supplier(&supplier.id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, supplier.name);
    }

    #[test]
    fn test_get_supplier_not_found() {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);

        let result = service.get_supplier("UNKNOWN");
        assert!(result.is_none());
    }
}
