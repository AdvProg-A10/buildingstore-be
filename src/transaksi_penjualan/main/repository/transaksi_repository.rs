use std::collections::HashMap;

use crate::transaksi_penjualan::main::model::transaksi::Transaksi;

pub trait TransaksiRepository: Send + Sync {
    fn save(&self, transaksi: Transaksi) -> Result<Transaksi, String>;
    fn find_by_id(&self, id: &str) -> Option<Transaksi>;
    fn find_all(&self, filters: Option<HashMap<String, String>>) -> Vec<Transaksi>;
    fn delete(&self, id: &str) -> Result<(), String>;
    fn update(&self, transaksi: Transaksi) -> Result<Transaksi, String>;
}
