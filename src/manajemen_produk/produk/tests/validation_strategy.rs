use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::validation::{ValidationRule, NamaNotEmpty, HargaNonNegatif};

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