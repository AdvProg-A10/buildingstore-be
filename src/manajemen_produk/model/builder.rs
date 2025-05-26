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

