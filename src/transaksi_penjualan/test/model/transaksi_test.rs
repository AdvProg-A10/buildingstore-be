#[cfg(test)]
mod tests {
    use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
    use crate::transaksi_penjualan::main::model::transaksi::{Transaksi, DetailProdukTransaksi};

    #[test]
    fn test_buat_transaksi_baru() {
        let transaksi = Transaksi::buat_transaksi_baru(
            "kasir123".to_string(),
            "pelanggan123".to_string(),
            vec![
                DetailProdukTransaksi {
                    produk_id: "produk1".to_string(),
                    nama_produk: "Kopi".to_string(),
                    jumlah: 2,
                    harga_satuan: 10000.0,
                },
                DetailProdukTransaksi {
                    produk_id: "produk2".to_string(),
                    nama_produk: "Roti".to_string(),
                    jumlah: 1,
                    harga_satuan: 5000.0,
                },
            ],
        );

        assert_eq!(transaksi.kasir_id, "kasir123");
        assert_eq!(transaksi.pelanggan_id, "pelanggan123");
        assert_eq!(transaksi.status, StatusTransaksi::MasihDiproses);
        assert_eq!(transaksi.total_harga(), 25000.0);
    }
}