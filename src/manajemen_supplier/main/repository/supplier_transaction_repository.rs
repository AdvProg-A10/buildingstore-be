use crate::manajemen_supplier::main::model::supplier_transaction::SupplierTransaction;

pub trait SupplierTransactionRepository: Send + Sync {
    fn save(&self, transaction: SupplierTransaction) -> Result<SupplierTransaction, String>;
    fn find_by_id(&self, id: &str) -> Option<SupplierTransaction>;
    fn find_by_supplier_id(&self, supplier_id: &str) -> Vec<SupplierTransaction>;
}
