use crate::manajemen_supplier::main::model::supplier::Supplier;

pub trait SupplierService: Send + Sync {
    fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String>;
    fn update_supplier(&self, supplier: Supplier) -> Result<(), String>;
    fn delete_supplier(&self, id: &str) -> Result<(), String>;
    fn get_supplier(&self, id: &str) -> Option<Supplier>;
}
