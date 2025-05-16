use crate::manajemen_supplier::main::model::supplier::Supplier;

pub trait SupplierNotifier: Send + Sync {
    fn notify_supplier_saved(&self, supplier: &Supplier);
}