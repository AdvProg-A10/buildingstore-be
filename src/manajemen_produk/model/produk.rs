// Struct yang merepresentasikan entitas produk dalam sistem.

// # Fields
// - `id`: ID unik produk (opsional, None untuk produk baru)
// - `nama`: Nama produk (wajib)
// - `kategori`: Kategori produk (wajib)
// - `harga`: Harga produk dalam bentuk float (wajib)
// - `stok`: Jumlah stok tersedia (wajib)
// - `deskripsi`: Deskripsi tambahan produk (opsional)

// # Methods
// - `with_id()`: Constructor untuk produk yang sudah ada di database
// - `new()`: Constructor untuk produk baru
// - `validate()`: Validasi data produk sebelum disimpan

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Produk {
    pub id: Option<i64>,
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

impl Produk {
    pub fn with_id(
        id: i64,
        nama: String,
        kategori: String,
        harga: f64,
        stok: u32,
        deskripsi: Option<String>,
    ) -> Self {
        Self {
            id: Some(id),
            nama,
            kategori,
            harga,
            stok,
            deskripsi,
        }
    }
    
    pub fn new(
        nama: String,
        kategori: String,
        harga: f64,
        stok: u32,
        deskripsi: Option<String>,
    ) -> Self {
        Self {
            id: None,
            nama,
            kategori,
            harga,
            stok,
            deskripsi,
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        use crate::manajemen_produk::validation::ProdukValidator;
        
        let validator = ProdukValidator::default();
        validator.validate(self)
    }
}

fn setup_test_products() -> Vec<Produk> {
    vec![
        Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        ),
        Produk::new(
            "Cat Tembok".to_string(),
            "Material".to_string(),
            150_000.0,
            50,
            Some("Cat tembok anti air".to_string()),
        ),
        Produk::new(
            "Smartphone".to_string(),
            "Elektronik".to_string(),
            8_000_000.0,
            20,
            Some("Smartphone dengan kamera 108MP".to_string()),
        ),
    ]
}

