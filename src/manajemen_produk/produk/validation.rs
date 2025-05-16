use super::model::Produk;

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

pub struct ProdukValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ProdukValidator {
    pub fn default() -> Self {
        Self {
            rules: vec![
                Box::new(NamaNotEmpty),
                Box::new(HargaNonNegatif),
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
