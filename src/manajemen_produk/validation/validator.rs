use crate::manajemen_produk::model::Produk;
use super::rules::{
    ValidationRule,
    NamaNotEmpty,
    KategoriNotEmpty,
    HargaNonNegatif,
    StokNonNegatif,
    DeskripsiMaxLength,
};

pub struct ProdukValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ProdukValidator {
    pub fn default() -> Self {
        Self {
            rules: vec![
                Box::new(NamaNotEmpty),
                Box::new(KategoriNotEmpty),
                Box::new(HargaNonNegatif),
                Box::new(StokNonNegatif),
                Box::new(DeskripsiMaxLength),
            ],
        }
    }

    pub fn validate(&self, produk: &Produk) -> Result<(), Vec<String>> {
        let errors = self.rules
            .iter()
            .filter_map(|rule| rule.validate(produk).err())
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[test]
fn test_validation_edge_cases() {
    // Edge case: product with minimum valid values
    let min_produk = Produk::new(
        "A".into(),  // Minimum name length
        "B".into(),  // Minimum category length
        0.0,         // Minimum price
        0,           // Minimum stock
        None,        // No description
    );
    assert!(min_produk.validate().is_ok());
    
    // Edge case: product with very large values
    let max_produk = Produk::new(
        "Very Long Product Name".into(),
        "Very Long Category Name".into(),
        f64::MAX,     // Maximum float value
        u32::MAX,     // Maximum u32 value
        Some("a".repeat(500)),  // Maximum description length
    );
    assert!(max_produk.validate().is_ok());
}

#[test]
fn test_produk_validation_all_rules() {
    // Create a valid product
    let valid_produk = Produk::new(
        "Laptop Gaming".into(),
        "Elektronik".into(),
        15_000_000.0,
        10,
        Some("Laptop dengan spesifikasi tinggi".into())
    );
    
    // Test all validation rules
    assert!(valid_produk.validate().is_ok());
    
    // Create an invalid product with multiple validation failures
    let invalid_produk = Produk::new(
        "".into(),
        "".into(),
        -1000.0,
        10,
        Some("a".repeat(501))
    );
    
    // Test all validation rules
    let validation_result = invalid_produk.validate();
    assert!(validation_result.is_err());
    
    // Check for all expected error messages
    let error_messages = validation_result.unwrap_err();
    assert!(error_messages.contains(&"Nama produk tidak boleh kosong".to_string()));
    assert!(error_messages.contains(&"Kategori produk tidak boleh kosong".to_string()));
    assert!(error_messages.contains(&"Harga tidak boleh negatif".to_string()));
    assert!(error_messages.contains(&"Deskripsi terlalu panjang (maksimal 500 karakter)".to_string()));
}