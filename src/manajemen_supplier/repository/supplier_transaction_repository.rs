use async_trait::async_trait;
use sqlx::{Any, pool::PoolConnection};
use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
use mockall::automock;


#[automock]
#[async_trait]
pub trait SupplierTransactionRepository: Send + Sync {
    async fn save(&self, transaction: SupplierTransaction, db: PoolConnection<Any>) -> Result<SupplierTransaction, sqlx::Error>;
    async fn find_by_id(&self, id: &str, db: PoolConnection<Any>) -> Result<SupplierTransaction, sqlx::Error>;
    async fn find_by_supplier_id(&self, supplier_id: &str, db: PoolConnection<Any>) -> Result<Vec<SupplierTransaction>, sqlx::Error>;
    async fn find_all(&self, db: PoolConnection<Any>) -> Result<Vec<SupplierTransaction>, sqlx::Error>;
}