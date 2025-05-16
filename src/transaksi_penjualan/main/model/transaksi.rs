use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;

#[derive(Debug, Clone)]
pub struct Transaksi {
    pub id: String,
    pub waktu: DateTime<Utc>,
    pub kasir_id: String,
    pub pelanggan_id: String,
    pub produk: Vec<DetailProdukTransaksi>,
    pub status: StatusTransaksi,
}

impl Transaksi {
    pub fn buat_transaksi_baru( 
        kasir_id: String,
        pelanggan_id: String,
        produk: Vec<DetailProdukTransaksi>,
    ) -> Self {
        Transaksi {
            id: format!("TRX-{}", Uuid::new_v4()),
            waktu: Utc::now(),
            kasir_id,
            pelanggan_id,
            produk,
            status: StatusTransaksi::MasihDiproses,
        }
    }

    pub fn total_harga(&self) -> f64 {
        self.produk
            .iter()
            .map(|p| p.harga_satuan * p.jumlah as f64)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct DetailProdukTransaksi {
    pub produk_id: String,
    pub nama_produk: String,
    pub jumlah: u32,
    pub harga_satuan: f64,
}
