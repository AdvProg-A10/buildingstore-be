use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
use crate::transaksi_penjualan::main::model::transaksi::Transaksi;
use crate::transaksi_penjualan::main::repository::transaksi_repository::TransaksiRepository;

pub struct TransaksiRepositoryImpl {
    transaksi_map: Arc<Mutex<HashMap<String, Transaksi>>>,
}

impl TransaksiRepositoryImpl {
    pub fn new() -> Self {
        Self {
            transaksi_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl TransaksiRepository for TransaksiRepositoryImpl {
    fn save(&self, transaksi: Transaksi) -> Result<Transaksi, String> {
        let mut transaksi_map = self.transaksi_map.lock().unwrap();
        let transaksi_clone = transaksi.clone();
        transaksi_map.insert(transaksi.id.clone(), transaksi);
        Ok(transaksi_clone)
    }

    fn find_by_id(&self, id: &str) -> Option<Transaksi> {
        let transaksi_map = self.transaksi_map.lock().unwrap();
        transaksi_map.get(id).cloned()
    }

    fn find_all(&self, filters: Option<HashMap<String, String>>) -> Vec<Transaksi> {
        let transaksi_map = self.transaksi_map.lock().unwrap();
        let mut result: Vec<Transaksi> = transaksi_map.values().cloned().collect();

        if let Some(filter_map) = filters {
            if let Some(status_str) = filter_map.get("status") {
                if let Some(status_enum) = StatusTransaksi::from_string(status_str) {
                    result = result
                        .into_iter()
                        .filter(|trx| trx.status == status_enum)
                        .collect();
                }
            }

            if let Some(pelanggan_id) = filter_map.get("pelanggan_id") {
                result = result
                    .into_iter()
                    .filter(|trx| trx.pelanggan_id == *pelanggan_id)
                    .collect();
            }

            if let Some(kasir_id) = filter_map.get("kasir_id") {
                result = result
                    .into_iter()
                    .filter(|trx| trx.kasir_id == *kasir_id)
                    .collect();
            }
        }

        result
    }

    fn update(&self, transaksi: Transaksi) -> Result<Transaksi, String> {
        let mut transaksi_map = self.transaksi_map.lock().unwrap();
        if !transaksi_map.contains_key(&transaksi.id) {
            return Err(format!("Transaksi dengan ID {} tidak ditemukan", transaksi.id));
        }
        let transaksi_clone = transaksi.clone();
        transaksi_map.insert(transaksi.id.clone(), transaksi);
        Ok(transaksi_clone)
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let mut transaksi_map = self.transaksi_map.lock().unwrap();
        if transaksi_map.remove(id).is_none() {
            return Err(format!("Transaksi dengan ID {} tidak ditemukan", id));
        }
        Ok(())
    }
}
