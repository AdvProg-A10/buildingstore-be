#[cfg(test)]
mod transaksi_controller_tests {
    use crate::transaksi_penjualan::main::controller::transaksi_controller::TransaksiController;
    use crate::transaksi_penjualan::main::repository::transaksi_repository_impl::TransaksiRepositoryImpl;
    use crate::transaksi_penjualan::main::service::transaksi_service::TransaksiService;
    use std::sync::Arc;

    fn setup_controller() -> TransaksiController {
        let repo = Arc::new(TransaksiRepositoryImpl::new());
        let service = Arc::new(TransaksiService { repo });
        TransaksiController { service }
    }

    #[test]
    fn test_controller_buat_transaksi() {
        let controller = setup_controller();
        let result = controller.buat_transaksi(
            "kasirX".to_string(),
            "pelangganX".to_string(),
            vec![("p1".to_string(), "produkX".to_string(), 3, 123.45)],
        );

        assert!(result.is_ok());
        let transaksi = result.unwrap();
        assert_eq!(transaksi.kasir_id, "kasirX");
        assert_eq!(transaksi.produk.len(), 1);
        assert_eq!(transaksi.total_harga(), 370.35);
    }

    #[test]
    fn test_controller_lihat_transaksi_kosong() {
        let controller = setup_controller();
        let result = controller.lihat_transaksi(None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_controller_update_transaksi() {
        let controller = setup_controller();
        let transaksi = controller
            .buat_transaksi(
                "kasirY".to_string(),
                "pelangganY".to_string(),
                vec![("p1".to_string(), "produkY".to_string(), 1, 50.0)],
            )
            .unwrap();

        let update_result = controller.update_transaksi(
            &transaksi.id,
            vec![("p2".to_string(), "produkZ".to_string(), 2, 25.0)],
        );

        assert!(update_result.is_ok());
        let updated = update_result.unwrap();
        assert_eq!(updated.total_harga(), 50.0);
        assert_eq!(updated.produk.len(), 1);
        assert_eq!(updated.produk[0].nama_produk, "produkZ");
    }
}
