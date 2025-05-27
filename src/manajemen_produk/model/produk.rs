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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_produk_new_constructor() {
        let produk = Produk::new(
            "Test Product".to_string(),
            "Test Category".to_string(),
            100.0,
            10,
            Some("Test description".to_string()),
        );

        assert_eq!(produk.id, None);
        assert_eq!(produk.nama, "Test Product");
        assert_eq!(produk.kategori, "Test Category");
        assert_eq!(produk.harga, 100.0);
        assert_eq!(produk.stok, 10);
        assert_eq!(produk.deskripsi, Some("Test description".to_string()));
    }

    #[test]
    fn test_produk_new_constructor_without_description() {
        let produk = Produk::new(
            "Test Product".to_string(),
            "Test Category".to_string(),
            100.0,
            10,
            None,
        );

        assert_eq!(produk.id, None);
        assert_eq!(produk.nama, "Test Product");
        assert_eq!(produk.kategori, "Test Category");
        assert_eq!(produk.harga, 100.0);
        assert_eq!(produk.stok, 10);
        assert_eq!(produk.deskripsi, None);
    }

    #[test]
    fn test_produk_with_id_constructor() {
        let produk = Produk::with_id(
            1,
            "Test Product".to_string(),
            "Test Category".to_string(),
            100.0,
            10,
            Some("Test description".to_string()),
        );

        assert_eq!(produk.id, Some(1));
        assert_eq!(produk.nama, "Test Product");
        assert_eq!(produk.kategori, "Test Category");
        assert_eq!(produk.harga, 100.0);
        assert_eq!(produk.stok, 10);
        assert_eq!(produk.deskripsi, Some("Test description".to_string()));
    }

    #[test]
    fn test_produk_with_id_constructor_without_description() {
        let produk = Produk::with_id(
            2,
            "Test Product".to_string(),
            "Test Category".to_string(),
            100.0,
            10,
            None,
        );

        assert_eq!(produk.id, Some(2));
        assert_eq!(produk.nama, "Test Product");
        assert_eq!(produk.kategori, "Test Category");
        assert_eq!(produk.harga, 100.0);
        assert_eq!(produk.stok, 10);
        assert_eq!(produk.deskripsi, None);
    }

    #[test]
    fn test_produk_clone() {
        let original = Produk::new(
            "Original Product".to_string(),
            "Original Category".to_string(),
            200.0,
            20,
            Some("Original description".to_string()),
        );

        let cloned = original.clone();

        assert_eq!(original.id, cloned.id);
        assert_eq!(original.nama, cloned.nama);
        assert_eq!(original.kategori, cloned.kategori);
        assert_eq!(original.harga, cloned.harga);
        assert_eq!(original.stok, cloned.stok);
        assert_eq!(original.deskripsi, cloned.deskripsi);
    }

    #[test]
    fn test_produk_debug_format() {
        let produk = Produk::new(
            "Debug Product".to_string(),
            "Debug Category".to_string(),
            300.0,
            30,
            Some("Debug description".to_string()),
        );

        let debug_string = format!("{:?}", produk);
        assert!(debug_string.contains("Debug Product"));
        assert!(debug_string.contains("Debug Category"));
        assert!(debug_string.contains("300"));
        assert!(debug_string.contains("30"));
        assert!(debug_string.contains("Debug description"));
    }

    #[test]
    fn test_produk_validate_call() {
        let produk = Produk::new(
            "Valid Product".to_string(),
            "Valid Category".to_string(),
            100.0,
            10,
            Some("Valid description".to_string()),
        );

        // Test that validate method can be called
        // The actual validation logic is in the validator module
        let _result = produk.validate();
        // We don't assert the result since it depends on the validator implementation
    }

    #[test]
    fn test_setup_test_products() {
        let products = setup_test_products();
        
        assert_eq!(products.len(), 3);
        
        // Test first product - Laptop Gaming
        assert_eq!(products[0].nama, "Laptop Gaming");
        assert_eq!(products[0].kategori, "Elektronik");
        assert_eq!(products[0].harga, 15_000_000.0);
        assert_eq!(products[0].stok, 10);
        assert_eq!(products[0].deskripsi, Some("Laptop dengan RTX 4060".to_string()));
        assert_eq!(products[0].id, None);
        
        // Test second product - Cat Tembok
        assert_eq!(products[1].nama, "Cat Tembok");
        assert_eq!(products[1].kategori, "Material");
        assert_eq!(products[1].harga, 150_000.0);
        assert_eq!(products[1].stok, 50);
        assert_eq!(products[1].deskripsi, Some("Cat tembok anti air".to_string()));
        assert_eq!(products[1].id, None);
        
        // Test third product - Smartphone
        assert_eq!(products[2].nama, "Smartphone");
        assert_eq!(products[2].kategori, "Elektronik");
        assert_eq!(products[2].harga, 8_000_000.0);
        assert_eq!(products[2].stok, 20);
        assert_eq!(products[2].deskripsi, Some("Smartphone dengan kamera 108MP".to_string()));
        assert_eq!(products[2].id, None);
    }

    #[test]
    fn test_produk_with_special_characters() {
        let special_chars = "特殊字符 🚀 émojis & symbols!@#$%^&*()";
        let produk = Produk::new(
            special_chars.to_string(),
            special_chars.to_string(),
            100.0,
            10,
            Some(special_chars.to_string()),
        );

        assert_eq!(produk.nama, special_chars);
        assert_eq!(produk.kategori, special_chars);
        assert_eq!(produk.deskripsi, Some(special_chars.to_string()));
    }

    #[test]
    fn test_produk_with_very_long_strings() {
        let long_string = "a".repeat(1000);
        let produk = Produk::new(
            long_string.clone(),
            long_string.clone(),
            100.0,
            10,
            Some(long_string.clone()),
        );

        assert_eq!(produk.nama.len(), 1000);
        assert_eq!(produk.kategori.len(), 1000);
        assert_eq!(produk.deskripsi.as_ref().unwrap().len(), 1000);
    }

    #[test]
    fn test_produk_with_zero_values() {
        let produk = Produk::new(
            "Zero Test".to_string(),
            "Test Category".to_string(),
            0.0,
            0,
            None,
        );

        assert_eq!(produk.harga, 0.0);
        assert_eq!(produk.stok, 0);
        assert_eq!(produk.deskripsi, None);
    }

    #[test]
    fn test_produk_with_negative_values() {
        let produk = Produk::with_id(
            -1,
            "Negative Test".to_string(),
            "Test Category".to_string(),
            -100.0,
            0, // u32 can't be negative
            None,
        );

        assert_eq!(produk.id, Some(-1));
        assert_eq!(produk.harga, -100.0);
    }

    #[test]
    fn test_produk_with_maximum_values() {
        let produk = Produk::with_id(
            i64::MAX,
            "Max Test".to_string(),
            "Test Category".to_string(),
            f64::MAX,
            u32::MAX,
            Some("Max description".to_string()),
        );

        assert_eq!(produk.id, Some(i64::MAX));
        assert_eq!(produk.harga, f64::MAX);
        assert_eq!(produk.stok, u32::MAX);
    }

    #[test]
    fn test_produk_with_empty_strings() {
        let produk = Produk::new(
            "".to_string(),
            "".to_string(),
            100.0,
            10,
            Some("".to_string()),
        );

        assert_eq!(produk.nama, "");
        assert_eq!(produk.kategori, "");
        assert_eq!(produk.deskripsi, Some("".to_string()));
    }

    #[test]
    fn test_produk_field_access() {
        let produk = Produk::with_id(
            42,
            "Access Test".to_string(),
            "Access Category".to_string(),
            250.5,
            15,
            Some("Access description".to_string()),
        );

        // Test direct field access (all fields are public)
        assert_eq!(produk.id.unwrap(), 42);
        assert_eq!(produk.nama.as_str(), "Access Test");
        assert_eq!(produk.kategori.as_str(), "Access Category");
        assert_eq!(produk.harga, 250.5);
        assert_eq!(produk.stok, 15);
        assert_eq!(produk.deskripsi.as_ref().unwrap().as_str(), "Access description");
    }
}