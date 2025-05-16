use crate::transaksi_penjualan::main::model::transaksi::Transaksi;
use crate::transaksi_penjualan::main::patterns::state::TransaksiStateContext;

pub trait TransaksiObserver: Send + Sync {
    fn on_transaksi_selesai(&self, transaksi: &Transaksi);
    fn on_transaksi_dibatalkan(&self, transaksi: &Transaksi);
}

pub trait StokObserver: TransaksiObserver {

}

pub struct TransaksiSubject {
    state_context: TransaksiStateContext,
    observers: Vec<Box<dyn TransaksiObserver>>,
}

impl TransaksiSubject {
    pub fn new(transaksi: Transaksi) -> Self {
        TransaksiSubject {
            state_context: TransaksiStateContext::new(transaksi),
            observers: Vec::new(),
        }
    }
    
    pub fn attach(&mut self, observer: Box<dyn TransaksiObserver>) {
        self.observers.push(observer);
    }
    
    pub fn detach(&mut self, index: usize) {
        if index < self.observers.len() {
            self.observers.remove(index);
        }
    }
    
    pub fn update_produk(&self, produk_baru: Vec<crate::transaksi_penjualan::main::model::transaksi::DetailProdukTransaksi>) -> Result<Transaksi, String> {
        self.state_context.update_produk(produk_baru)
    }
    
    pub fn selesaikan_transaksi(&mut self) -> Result<Transaksi, String> {
        let result = self.state_context.selesaikan_transaksi();
        if let Ok(transaksi) = &result {
            // Notify all observers that transaction is complete
            for observer in &self.observers {
                observer.on_transaksi_selesai(transaksi);
            }
        }
        result
    }
    
    pub fn batalkan_transaksi(&mut self) -> Result<Transaksi, String> {
        let result = self.state_context.batalkan_transaksi();
        if let Ok(transaksi) = &result {
            // Notify all observers that transaction is canceled
            for observer in &self.observers {
                observer.on_transaksi_dibatalkan(transaksi);
            }
        }
        result
    }
}
