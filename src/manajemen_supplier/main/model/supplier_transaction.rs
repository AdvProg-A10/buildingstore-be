use chrono::{DateTime, Utc};
#[derive(Debug, Clone, PartialEq)]
pub struct SupplierTransaction {
    pub id: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub pengiriman_info: String,
    pub tanggal_transaksi: DateTime<Utc>,
}