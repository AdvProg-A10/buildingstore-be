// Builder untuk membuat instance Produk dengan cara yang lebih fleksibel dan ergonomis.
// Menggunakan pola builder untuk memungkinkan pembuatan Produk secara bertahap.

// # Methods
// - `new()`: Membuat builder baru dengan nama dan kategori (field wajib)
// - `id()`: Menetapkan ID produk (opsional)
// - `harga()`: Menetapkan harga produk
// - `stok()`: Menetapkan stok produk
// - `deskripsi()`: Menetapkan deskripsi produk (opsional)
// - `build()`: Membuat Produk dan memvalidasinya, mengembalikan Result

use crate::manajemen_produk::model::Produk;

pub struct ProdukBuilder {
    id: Option<i64>,
    nama: String,
    kategori: String,
    harga: f64,
    stok: u32,
    deskripsi: Option<String>,
}

impl ProdukBuilder {
    pub fn new(nama: String, kategori: String) -> Self {
        Self {
            id: None,
            nama,
            kategori,
            harga: 0.0,
            stok: 0,
            deskripsi: None,
        }
    }
    
    pub fn id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }
    
    pub fn harga(mut self, harga: f64) -> Self {
        self.harga = harga;
        self
    }
    
    pub fn stok(mut self, stok: u32) -> Self {
        self.stok = stok;
        self
    }
    
    pub fn deskripsi(mut self, deskripsi: String) -> Self {
        self.deskripsi = Some(deskripsi);
        self
    }
    
    pub fn build(self) -> Result<Produk, Vec<String>> {
        let produk = Produk {
            id: self.id,
            nama: self.nama,
            kategori: self.kategori,
            harga: self.harga,
            stok: self.stok,
            deskripsi: self.deskripsi,
        };
        
        match produk.validate() {
            Ok(_) => Ok(produk),
            Err(errors) => Err(errors),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_produk_builder() {
        // Using the builder pattern
        let produk_result = ProdukBuilder::new("Laptop Gaming".to_string(), "Elektronik".to_string())
            .harga(15_000_000.0)
            .stok(10)
            .deskripsi("Laptop dengan RTX 4060".to_string())
            .build();
        
        assert!(produk_result.is_ok());
        let produk = produk_result.unwrap();
        
        assert_eq!(produk.nama, "Laptop Gaming");
        assert_eq!(produk.kategori, "Elektronik");
        assert_eq!(produk.harga, 15_000_000.0);
        assert_eq!(produk.stok, 10);
        assert_eq!(produk.deskripsi, Some("Laptop dengan RTX 4060".to_string()));
    }

    #[test]
    fn test_builder_validation() {
        // Test validation with empty name
        let produk_result = ProdukBuilder::new("".to_string(), "Elektronik".to_string())
            .harga(15_000_000.0)
            .stok(10)
            .build();
        
        assert!(produk_result.is_err());
        assert_eq!(produk_result.unwrap_err(), vec!["Nama produk tidak boleh kosong"]);
        
        // Test validation with negative price
        let produk_result = ProdukBuilder::new("Laptop Gaming".to_string(), "Elektronik".to_string())
            .harga(-5000.0)
            .stok(10)
            .build();
        
        assert!(produk_result.is_err());
        assert_eq!(produk_result.unwrap_err(), vec!["Harga tidak boleh negatif"]);
    }

    #[test]
    fn test_builder_with_id() {
        let produk_result = ProdukBuilder::new("Mouse Gaming".to_string(), "Elektronik".to_string())
            .id(100)
            .harga(500_000.0)
            .stok(25)
            .build();
        
        assert!(produk_result.is_ok());
        let produk = produk_result.unwrap();
        assert_eq!(produk.id, Some(100));
    }

    #[test]
    fn test_builder_minimal_fields() {
        let produk_result = ProdukBuilder::new("Produk Test".to_string(), "Test".to_string())
            .harga(1000.0)
            .stok(1)
            .build();
        
        assert!(produk_result.is_ok());
        let produk = produk_result.unwrap();
        assert_eq!(produk.nama, "Produk Test");
        assert_eq!(produk.kategori, "Test");
        assert_eq!(produk.deskripsi, None);
        assert_eq!(produk.id, None);
    }
}