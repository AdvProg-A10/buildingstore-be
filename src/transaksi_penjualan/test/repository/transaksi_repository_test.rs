#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::transaksi_penjualan::main::model::transaksi::{DetailProdukTransaksi, Transaksi};
    use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
    use crate::transaksi_penjualan::main::repository::transaksi_repository::TransaksiRepository;
    use crate::transaksi_penjualan::main::repository::transaksi_repository_impl::TransaksiRepositoryImpl;

    #[test]
    fn test_simpan_transaksi() {
        let repo = TransaksiRepositoryImpl::new();
        let detail = DetailProdukTransaksi {
            produk_id: "PRD-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 3,
            harga_satuan: 15000.0,
        };

        let transaksi = Transaksi {
            id: format!("TRX-{}", Uuid::new_v4()),
            pelanggan_id: "PLG-001".to_string(),
            kasir_id: "KSR-001".to_string(),
            waktu: Utc::now(),
            produk: vec![detail.clone()],
            status: StatusTransaksi::MasihDiproses,
        };

        let result = repo.save(transaksi.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, transaksi.id);
    }

    #[test]
    fn test_cari_transaksi_berdasarkan_id() {
        let repo = TransaksiRepositoryImpl::new();
        let transaksi_id = format!("TRX-{}", Uuid::new_v4());

        let transaksi = Transaksi {
            id: transaksi_id.clone(),
            pelanggan_id: "PLG-001".to_string(),
            kasir_id: "KSR-001".to_string(),
            waktu: Utc::now(),
            produk: vec![],
            status: StatusTransaksi::MasihDiproses,
        };

        repo.save(transaksi.clone()).unwrap();
        let result = repo.find_by_id(&transaksi_id);

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, transaksi_id);
    }

    #[test]
    fn test_update_transaksi() {
        let repo = TransaksiRepositoryImpl::new();
        let transaksi_id = format!("TRX-{}", Uuid::new_v4());

        let mut transaksi = Transaksi {
            id: transaksi_id.clone(),
            pelanggan_id: "PLG-001".to_string(),
            kasir_id: "KSR-001".to_string(),
            waktu: Utc::now(),
            produk: vec![],
            status: StatusTransaksi::MasihDiproses,
        };

        repo.save(transaksi.clone()).unwrap();

        transaksi.status = StatusTransaksi::Selesai;
        let updated = repo.update(transaksi.clone()).unwrap();

        assert_eq!(updated.status, StatusTransaksi::Selesai);
    }

    #[test]
    fn test_hapus_transaksi() {
        let repo = TransaksiRepositoryImpl::new();
        let transaksi_id = format!("TRX-{}", Uuid::new_v4());

        let transaksi = Transaksi {
            id: transaksi_id.clone(),
            pelanggan_id: "PLG-001".to_string(),
            kasir_id: "KSR-001".to_string(),
            waktu: Utc::now(),
            produk: vec![],
            status: StatusTransaksi::MasihDiproses,
        };

        repo.save(transaksi.clone()).unwrap();
        let result = repo.delete(&transaksi_id);

        assert!(result.is_ok());
        assert!(repo.find_by_id(&transaksi_id).is_none());
    }
}