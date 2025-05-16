#[cfg(test)]
mod tests { 
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
    use crate::transaksi_penjualan::main::model::transaksi::{DetailProdukTransaksi, Transaksi};
    use crate::transaksi_penjualan::main::patterns::{StokObserver, TransaksiObserver, TransaksiSubject};

    struct MockStokManager {
        stok_dikurangi: Arc<Mutex<bool>>,
        stok_dikembalikan: Arc<Mutex<bool>>,
    }

    impl TransaksiObserver for MockStokManager {
        fn on_transaksi_selesai(&self, _transaksi: &Transaksi) {
            *self.stok_dikurangi.lock().unwrap() = true;
        }
        
        fn on_transaksi_dibatalkan(&self, _transaksi: &Transaksi) {
            *self.stok_dikembalikan.lock().unwrap() = true;
        }
    }

    impl StokObserver for MockStokManager {}

    #[test]
    fn test_notify_observers_when_transaksi_selesai() {
        let stok_dikurangi = Arc::new(Mutex::new(false));
        let stok_dikembalikan = Arc::new(Mutex::new(false));
        
        let mock_stok_manager = MockStokManager {
            stok_dikurangi: stok_dikurangi.clone(),
            stok_dikembalikan: stok_dikembalikan.clone(),
        };
        
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
        
        let mut transaksi_subject = TransaksiSubject::new(transaksi);
        transaksi_subject.attach(Box::new(mock_stok_manager));
        
        let updated_transaksi = transaksi_subject.selesaikan_transaksi().unwrap();
        
        assert_eq!(updated_transaksi.status, StatusTransaksi::Selesai);
        assert_eq!(*stok_dikurangi.lock().unwrap(), true);
        assert_eq!(*stok_dikembalikan.lock().unwrap(), false);
    }

    #[test]
    fn test_notify_observers_when_transaksi_dibatalkan() {
        let stok_dikurangi = Arc::new(Mutex::new(false));
        let stok_dikembalikan = Arc::new(Mutex::new(false));
        
        let mock_stok_manager = MockStokManager {
            stok_dikurangi: stok_dikurangi.clone(),
            stok_dikembalikan: stok_dikembalikan.clone(),
        };
        
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
        
        let mut transaksi_subject = TransaksiSubject::new(transaksi);
        transaksi_subject.attach(Box::new(mock_stok_manager));
        
        let updated_transaksi = transaksi_subject.batalkan_transaksi().unwrap();
        
        assert_eq!(updated_transaksi.status, StatusTransaksi::Dibatalkan);
        assert_eq!(*stok_dikurangi.lock().unwrap(), false);
        assert_eq!(*stok_dikembalikan.lock().unwrap(), true);
    }
}
