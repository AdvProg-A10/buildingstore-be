use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;
use crate::transaksi_penjualan::main::model::transaksi::{DetailProdukTransaksi, Transaksi};

pub trait TransaksiState {
    fn update_produk(&self, transaksi: &Transaksi, produk_baru: Vec<DetailProdukTransaksi>) -> Result<Transaksi, String>;
    fn selesaikan_transaksi(&self, transaksi: &Transaksi) -> Result<Transaksi, String>;
    fn batalkan_transaksi(&self, transaksi: &Transaksi) -> Result<Transaksi, String>;
}

pub struct MasihDiprosesState;
pub struct SelesaiState;
pub struct DibatalkanState;

pub struct TransaksiStateContext {
    transaksi: Transaksi,
    state: Box<dyn TransaksiState>,
}

impl TransaksiStateContext {
    pub fn new(transaksi: Transaksi) -> Self {
        let state: Box<dyn TransaksiState> = match transaksi.status {
            StatusTransaksi::MasihDiproses => Box::new(MasihDiprosesState),
            StatusTransaksi::Selesai => Box::new(SelesaiState),
            StatusTransaksi::Dibatalkan => Box::new(DibatalkanState),
        };
        
        TransaksiStateContext { transaksi, state }
    }
    
    pub fn update_produk(&self, produk_baru: Vec<DetailProdukTransaksi>) -> Result<Transaksi, String> {
        self.state.update_produk(&self.transaksi, produk_baru)
    }
    
    pub fn selesaikan_transaksi(&mut self) -> Result<Transaksi, String> {
        let result = self.state.selesaikan_transaksi(&self.transaksi);
        if let Ok(updated_transaksi) = &result {
            self.transaksi = updated_transaksi.clone();
            self.state = Box::new(SelesaiState);
        }
        result
    }
    
    pub fn batalkan_transaksi(&mut self) -> Result<Transaksi, String> {
        let result = self.state.batalkan_transaksi(&self.transaksi);
        if let Ok(updated_transaksi) = &result {
            self.transaksi = updated_transaksi.clone();
            self.state = Box::new(DibatalkanState);
        }
        result
    }
}

impl TransaksiState for MasihDiprosesState {
    fn update_produk(&self, transaksi: &Transaksi, produk_baru: Vec<DetailProdukTransaksi>) -> Result<Transaksi, String> {
        let mut updated = transaksi.clone();
        updated.produk = produk_baru;
        Ok(updated)
    }
    
    fn selesaikan_transaksi(&self, transaksi: &Transaksi) -> Result<Transaksi, String> {
        let mut updated = transaksi.clone();
        updated.status = StatusTransaksi::Selesai;
        Ok(updated)
    }
    
    fn batalkan_transaksi(&self, transaksi: &Transaksi) -> Result<Transaksi, String> {
        let mut updated = transaksi.clone();
        updated.status = StatusTransaksi::Dibatalkan;
        Ok(updated)
    }
}

impl TransaksiState for SelesaiState {
    fn update_produk(&self, _transaksi: &Transaksi, _produk_baru: Vec<DetailProdukTransaksi>) -> Result<Transaksi, String> {
        Err("Tidak dapat mengubah transaksi dengan status Selesai".to_string())
    }
    
    fn selesaikan_transaksi(&self, transaksi: &Transaksi) -> Result<Transaksi, String> {
        Ok(transaksi.clone())
    }
    
    fn batalkan_transaksi(&self, _transaksi: &Transaksi) -> Result<Transaksi, String> {
        Err("Tidak dapat membatalkan transaksi dengan status Selesai".to_string())
    }
}

impl TransaksiState for DibatalkanState {
    fn update_produk(&self, _transaksi: &Transaksi, _produk_baru: Vec<DetailProdukTransaksi>) -> Result<Transaksi, String> {
        Err("Tidak dapat mengubah transaksi dengan status Dibatalkan".to_string())
    }
    
    fn selesaikan_transaksi(&self, _transaksi: &Transaksi) -> Result<Transaksi, String> {
        Err("Tidak dapat menyelesaikan transaksi dengan status Dibatalkan".to_string())
    }
    
    fn batalkan_transaksi(&self, transaksi: &Transaksi) -> Result<Transaksi, String> {
        Ok(transaksi.clone())
    }
}
