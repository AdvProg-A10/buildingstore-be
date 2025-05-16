#[cfg(test)]
mod transaksi_service_tests {
    use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
    use crate::transaksi_penjualan::main::repository::transaksi_repository_impl::TransaksiRepositoryImpl;
    use crate::transaksi_penjualan::main::service::transaksi_service::TransaksiService;
    use std::sync::Arc;

    fn setup_service() -> TransaksiService {
        let repo = Arc::new(TransaksiRepositoryImpl::new());
        TransaksiService { repo }
    }

    #[test]
    fn test_buat_transaksi() {
        let service = setup_service();
        let result = service.buat_transaksi(
            "kasir1".to_string(),
            "pelanggan1".to_string(),
            vec![
                ("p1".to_string(), "produk1".to_string(), 2, 100.0),
                ("p2".to_string(), "produk2".to_string(), 1, 150.0),
            ],
        );

        assert!(result.is_ok());
        let transaksi = result.unwrap();
        assert_eq!(transaksi.kasir_id, "kasir1");
        assert_eq!(transaksi.pelanggan_id, "pelanggan1");
        assert_eq!(transaksi.produk.len(), 2);
        assert_eq!(transaksi.total_harga(), 350.0);
    }

    #[test]
    fn test_update_transaksi_gagal_setelah_selesai() {
        let service = setup_service();
        let mut transaksi = service
            .buat_transaksi(
                "kasir2".to_string(),
                "pelanggan2".to_string(),
                vec![("p3".to_string(), "produk3".to_string(), 1, 200.0)],
            )
            .unwrap();

        transaksi.status = StatusTransaksi::Selesai;
        let _ = service.repo.update(transaksi.clone());

        let result = service.update_transaksi(
            &transaksi.id,
            vec![("p4".to_string(), "produk4".to_string(), 2, 50.0)],
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Transaksi sudah selesai dan tidak bisa diubah"
        );
    }

    #[test]
    fn test_batalkan_transaksi() {
        let service = setup_service();
        let transaksi = service
            .buat_transaksi(
                "kasir3".to_string(),
                "pelanggan3".to_string(),
                vec![("p5".to_string(), "produk5".to_string(), 1, 75.0)],
            )
            .unwrap();

        let result = service.batalkan_transaksi(&transaksi.id);
        assert!(result.is_ok());

        let updated = service.repo.find_by_id(&transaksi.id).unwrap();
        assert_eq!(updated.status, StatusTransaksi::Dibatalkan);
    }
}
