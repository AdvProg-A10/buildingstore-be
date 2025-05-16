#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::vec;

    use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
    use crate::transaksi_penjualan::main::model::transaksi::{DetailProdukTransaksi, Transaksi};
    use crate::transaksi_penjualan::main::patterns::state::TransaksiStateContext;

    #[test]
    fn test_masih_diproses_state_can_update() {
        let detail_produk = DetailProdukTransaksi {
            produk_id: "P-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 2,
            harga_satuan: 10000.0,
        };
        
        let transaksi = Transaksi {
            id: "TRX-001".to_string(),
            waktu: Utc::now(),
            kasir_id: "KSR-001".to_string(),
            pelanggan_id: "PLG-001".to_string(),
            produk: vec![detail_produk],
            status: StatusTransaksi::MasihDiproses,
        };
        
        let state_context = TransaksiStateContext::new(transaksi);
        
        let new_detail_produk = DetailProdukTransaksi {
            produk_id: "P-002".to_string(),
            nama_produk: "Produk B".to_string(),
            jumlah: 1,
            harga_satuan: 5000.0,
        };
        
        let result = state_context.update_produk(vec![new_detail_produk]);
        
        assert!(result.is_ok());
        let updated_transaksi = result.unwrap();
        assert_eq!(updated_transaksi.produk.len(), 1);
        assert_eq!(updated_transaksi.produk[0].produk_id, "P-002");
    }

    #[test]
    fn test_selesai_state_cannot_update() {
        let detail_produk = DetailProdukTransaksi {
            produk_id: "P-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 2,
            harga_satuan: 10000.0,
        };
        
        let transaksi = Transaksi {
            id: "TRX-001".to_string(),
            waktu: Utc::now(),
            kasir_id: "KSR-001".to_string(),
            pelanggan_id: "PLG-001".to_string(),
            produk: vec![detail_produk],
            status: StatusTransaksi::Selesai,
        };
        
        let state_context = TransaksiStateContext::new(transaksi);
        
        let new_detail_produk = DetailProdukTransaksi {
            produk_id: "P-002".to_string(),
            nama_produk: "Produk B".to_string(),
            jumlah: 1,
            harga_satuan: 5000.0,
        };
        
        let result = state_context.update_produk(vec![new_detail_produk]);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Tidak dapat mengubah transaksi dengan status Selesai"
        );
    }

    #[test]
    fn test_dibatalkan_state_cannot_update() {
        let detail_produk = DetailProdukTransaksi {
            produk_id: "P-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 2,
            harga_satuan: 10000.0,
        };
        
        let transaksi = Transaksi {
            id: "TRX-001".to_string(),
            waktu: Utc::now(),
            kasir_id: "KSR-001".to_string(),
            pelanggan_id: "PLG-001".to_string(),
            produk: vec![detail_produk],
            status: StatusTransaksi::Dibatalkan,
        };
        
        let state_context = TransaksiStateContext::new(transaksi);
        
        let new_detail_produk = DetailProdukTransaksi {
            produk_id: "P-002".to_string(),
            nama_produk: "Produk B".to_string(),
            jumlah: 1,
            harga_satuan: 5000.0,
        };
        
        let result = state_context.update_produk(vec![new_detail_produk]);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Tidak dapat mengubah transaksi dengan status Dibatalkan"
        );
    }

    #[test]
    fn test_change_status_from_masih_diproses_to_selesai() {
        let detail_produk = DetailProdukTransaksi {
            produk_id: "P-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 2,
            harga_satuan: 10000.0,
        };
        
        let transaksi = Transaksi {
            id: "TRX-001".to_string(),
            waktu: Utc::now(),
            kasir_id: "KSR-001".to_string(),
            pelanggan_id: "PLG-001".to_string(),
            produk: vec![detail_produk],
            status: StatusTransaksi::MasihDiproses,
        };
        
        let mut state_context = TransaksiStateContext::new(transaksi);
        
        let result = state_context.selesaikan_transaksi();
        
        assert!(result.is_ok());
        let updated_transaksi = result.unwrap();
        assert_eq!(updated_transaksi.status, StatusTransaksi::Selesai);
    }

    #[test]
    fn test_change_status_from_masih_diproses_to_dibatalkan() {
        let detail_produk = DetailProdukTransaksi {
            produk_id: "P-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 2,
            harga_satuan: 10000.0,
        };
        
        let transaksi = Transaksi {
            id: "TRX-001".to_string(),
            waktu: Utc::now(),
            kasir_id: "KSR-001".to_string(),
            pelanggan_id: "PLG-001".to_string(),
            produk: vec![detail_produk],
            status: StatusTransaksi::MasihDiproses,
        };
        
        let mut state_context = TransaksiStateContext::new(transaksi);
        
        let result = state_context.batalkan_transaksi();
        
        assert!(result.is_ok());
        let updated_transaksi = result.unwrap();
        assert_eq!(updated_transaksi.status, StatusTransaksi::Dibatalkan);
    }

    #[test]
    fn test_cannot_change_status_from_selesai_to_dibatalkan() {
        let detail_produk = DetailProdukTransaksi {
            produk_id: "P-001".to_string(),
            nama_produk: "Produk A".to_string(),
            jumlah: 2,
            harga_satuan: 10000.0,
        };
        
        let transaksi = Transaksi {
            id: "TRX-001".to_string(),
            waktu: Utc::now(),
            kasir_id: "KSR-001".to_string(),
            pelanggan_id: "PLG-001".to_string(),
            produk: vec![detail_produk],
            status: StatusTransaksi::Selesai,
        };
        
        let mut state_context = TransaksiStateContext::new(transaksi);
        
        let result = state_context.batalkan_transaksi();
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Tidak dapat membatalkan transaksi dengan status Selesai"
        );
    }
}
