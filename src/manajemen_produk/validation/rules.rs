use crate::manajemen_produk::model::Produk;

pub trait ValidationRule {
    fn validate(&self, produk: &Produk) -> Result<(), String>;
}

pub struct NamaNotEmpty;
impl ValidationRule for NamaNotEmpty {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        if produk.nama.trim().is_empty() {
            Err("Nama produk tidak boleh kosong".to_string())
        } else {
            Ok(())
        }
    }
}

pub struct HargaNonNegatif;
impl ValidationRule for HargaNonNegatif {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        if produk.harga < 0.0 {
            Err("Harga tidak boleh negatif".to_string())
        } else {
            Ok(())
        }
    }
}

pub struct KategoriNotEmpty;
impl ValidationRule for KategoriNotEmpty {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        if produk.kategori.trim().is_empty() {
            Err("Kategori produk tidak boleh kosong".to_string())
        } else {
            Ok(())
        }
    }
}

pub struct StokNonNegatif;
impl ValidationRule for StokNonNegatif {
    fn validate(&self, _produk: &Produk) -> Result<(), String> {
        // stok: u32 sudah tidak bisa negatif
        Ok(())
    }
}

pub struct DeskripsiMaxLength;
impl ValidationRule for DeskripsiMaxLength {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        const MAX_LENGTH: usize = 500;
        if let Some(desc) = &produk.deskripsi {
            if desc.len() > MAX_LENGTH {
                return Err("Deskripsi terlalu panjang (maksimal 500 karakter)".to_string());
            }
        }
        Ok(())
    }
}

#[test]
fn test_nama_not_empty() {
    let strategy = NamaNotEmpty;
    
    // Test invalid case
    let invalid_produk = Produk::new("".into(), "Elektronik".into(), 1000.0, 10, None);
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Nama produk tidak boleh kosong");
    
    // Test valid case
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&valid_produk).is_ok());
}

#[test]
fn test_kategori_not_empty() {
    let strategy = KategoriNotEmpty;
    
    // Test invalid case
    let invalid_produk = Produk::new("Laptop".into(), "".into(), 1000.0, 10, None);
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Kategori produk tidak boleh kosong");
    
    // Test valid case
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&valid_produk).is_ok());
}

#[test]
fn test_harga_non_negatif() {
    let strategy = HargaNonNegatif;
    
    // Test invalid case
    let invalid_produk = Produk::new("Laptop".into(), "Elektronik".into(), -1000.0, 10, None);
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Harga tidak boleh negatif");
    
    // Test valid case
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&valid_produk).is_ok());
    
    // Test edge case (zero price is valid)
    let zero_price_produk = Produk::new("Laptop".into(), "Elektronik".into(), 0.0, 10, None);
    assert!(strategy.validate(&zero_price_produk).is_ok());
}

#[test]
fn test_stok_non_negatif() {
    let strategy = StokNonNegatif;
    
    // Create a product with negative stock (this shouldn't be possible with u32, 
    // but we can test the validation logic)
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 0, None);
    assert!(strategy.validate(&valid_produk).is_ok());
}

#[test]
fn test_deskripsi_max_length() {
    let strategy = DeskripsiMaxLength;
    
    // Create a product with description longer than maximum
    let long_description = "a".repeat(501); // Assuming max length is 500
    let invalid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, Some(long_description));
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Deskripsi terlalu panjang (maksimal 500 karakter)");
    
    // Test valid case
    let valid_description = "a".repeat(500);
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, Some(valid_description));
    assert!(strategy.validate(&valid_produk).is_ok());
    
    // Test edge case (empty description is valid)
    let empty_desc_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, Some("".into()));
    assert!(strategy.validate(&empty_desc_produk).is_ok());
    
    // Test None description
    let none_desc_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&none_desc_produk).is_ok());
}

#[test]
fn test_nama_not_empty_strategy() {
    let strategy = NamaNotEmpty;
    let mut produk = Produk::new("".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&produk).is_err());

    produk.nama = "Laptop".into();
    assert!(strategy.validate(&produk).is_ok());
}

#[test]
fn test_harga_non_negatif_strategy() {
    let strategy = HargaNonNegatif;
    let mut produk = Produk::new("Laptop".into(), "Elektronik".into(), -1000.0, 10, None);
    assert!(strategy.validate(&produk).is_err());

    produk.harga = 1000.0;
    assert!(strategy.validate(&produk).is_ok());
}