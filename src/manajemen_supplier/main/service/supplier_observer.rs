use crate::manajemen_supplier::main::model::supplier::Supplier;
pub trait SupplierObserver: Send + Sync {
    fn on_supplier_saved(&self, supplier: &Supplier);
}