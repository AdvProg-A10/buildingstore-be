use chrono::{DateTime, Utc};

//sruct untuk supplier
#[derive(Debug, Clone, PartialEq)]
pub struct Supplier {
    pub id: String,
    pub name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub resi: String,
    pub updated_at: DateTime<Utc>
}