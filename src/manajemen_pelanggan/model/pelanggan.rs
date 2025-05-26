use chrono::{ Utc, NaiveDate };
use rocket::serde::{Serialize, Deserialize};

/// Struct representing a customer (Pelanggan) in the system.
/// Contains fields for ID, name, address, phone number, and join date.
/// 
/// The `new` method can be used to create a new `Pelanggan` with only the necessary fields.
/// ID and join date will be automatically initialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Pelanggan {
    pub id: i32,
    pub nama: String,
    pub alamat: String,
    pub no_telp: String,
    pub tanggal_gabung: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PelangganForm {
    pub nama: String,
    pub alamat: String,
    pub no_telp: String,
}

impl Pelanggan {
    /// Creates a new instance of `Pelanggan`. Automatically initializes the `id` to 0 
    /// and sets the `tanggal_gabung` to the current date. Use the default constructor
    /// to create a `Pelanggan` object from an existing data source.
    pub fn new(nama: String, alamat: String, no_telp: String) -> Self {
        Pelanggan {
            id: 0,
            nama,
            alamat,
            no_telp,
            tanggal_gabung: Utc::now().date_naive(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_pelanggan() {
        let pelanggan = Pelanggan::new( "Castorice".to_string(), "Amphoreus".to_string(), "1234567890".to_string());
        assert_eq!(pelanggan.nama, "Castorice");
        assert_eq!(pelanggan.alamat, "Amphoreus");
        assert_eq!(pelanggan.no_telp, "1234567890");
        assert_eq!(pelanggan.tanggal_gabung, Utc::now().date_naive());
    }
}