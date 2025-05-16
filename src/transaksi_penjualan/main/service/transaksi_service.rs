use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
use crate::transaksi_penjualan::main::model::transaksi::{DetailProdukTransaksi, Transaksi};
use crate::transaksi_penjualan::main::repository::transaksi_repository::TransaksiRepository;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TransaksiService {
    pub repo: Arc<dyn TransaksiRepository>,
}

impl TransaksiService {
    pub fn buat_transaksi(&self, kasir_id: String, pelanggan_id: String, produk_data: Vec<(String, String, u32, f64)>) -> Result<Transaksi, String> {
        let produk: Vec<DetailProdukTransaksi> = produk_data
            .into_iter()
            .map(|(id, nama, jumlah, harga)| DetailProdukTransaksi {
                produk_id: id,
                nama_produk: nama,
                jumlah,
                harga_satuan: harga,
            })
            .collect();

        let transaksi = Transaksi::buat_transaksi_baru(kasir_id, pelanggan_id, produk);
        self.repo.save(transaksi)
    }

    pub fn lihat_transaksi(&self, filters: Option<HashMap<String, String>>) -> Vec<Transaksi> {
        self.repo.find_all(filters)
    }

    pub fn update_transaksi(&self, transaksi_id: &str, produk_data: Vec<(String, String, u32, f64)>) -> Result<Transaksi, String> {
        let mut transaksi = self.repo.find_by_id(transaksi_id).ok_or("Transaksi tidak ditemukan")?;

        if transaksi.status != StatusTransaksi::MasihDiproses {
            return Err("Transaksi sudah selesai dan tidak bisa diubah".to_string());
        }

        let produk: Vec<DetailProdukTransaksi> = produk_data
            .into_iter()
            .map(|(id, nama, jumlah, harga)| DetailProdukTransaksi {
                produk_id: id,
                nama_produk: nama,
                jumlah,
                harga_satuan: harga,
            })
            .collect();

        transaksi.produk = produk;
        self.repo.update(transaksi)
    }

    pub fn batalkan_transaksi(&self, transaksi_id: &str) -> Result<(), String> {
        let mut transaksi = self.repo.find_by_id(transaksi_id).ok_or("Transaksi tidak ditemukan")?;

        if transaksi.status != StatusTransaksi::MasihDiproses {
            return Err("Transaksi sudah selesai dan tidak bisa dibatalkan".to_string());
        }

        transaksi.status = StatusTransaksi::Dibatalkan;
        self.repo.update(transaksi)?;
        Ok(())
    }
}
