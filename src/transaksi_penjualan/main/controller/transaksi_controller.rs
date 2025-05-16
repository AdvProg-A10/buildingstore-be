use crate::transaksi_penjualan::main::model::transaksi::Transaksi;
use crate::transaksi_penjualan::main::service::transaksi_service::TransaksiService;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TransaksiController {
    pub service: Arc<TransaksiService>,
}

impl TransaksiController {
    pub fn buat_transaksi(&self, kasir_id: String, pelanggan_id: String, produk: Vec<(String, String, u32, f64)>) -> Result<Transaksi, String> {
        self.service.buat_transaksi(kasir_id, pelanggan_id, produk)
    }

    pub fn lihat_transaksi(&self, filters: Option<HashMap<String, String>>) -> Vec<Transaksi> {
        self.service.lihat_transaksi(filters)
    }

    pub fn update_transaksi(&self, transaksi_id: &str, produk_baru: Vec<(String, String, u32, f64)>) -> Result<Transaksi, String> {
        self.service.update_transaksi(transaksi_id, produk_baru)
    }

    pub fn batalkan_transaksi(&self, transaksi_id: &str) -> Result<(), String> {
        self.service.batalkan_transaksi(transaksi_id)
    }
}