use crate::manajemen_supplier::main::model::supplier::Supplier;
pub trait SupplierRepository: Send + Sync {
    fn save(&self, supplier: Supplier) -> Result<Supplier, String>;
    fn find_by_id(&self, id: &str) -> Option<Supplier>;
    fn update(&self, supplier: Supplier) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}
