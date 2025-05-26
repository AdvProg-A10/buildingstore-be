use crate::manajemen_supplier::model::supplier::Supplier;
use::async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait SupplierNotifier: Send + Sync {
    async fn notify_supplier_saved(&self, supplier: &Supplier);
}