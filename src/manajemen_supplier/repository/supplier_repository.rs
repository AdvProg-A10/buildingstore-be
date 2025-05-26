use async_trait::async_trait;
use mockall::automock;
use crate::manajemen_supplier::model::supplier::Supplier;
use sqlx::{Any, pool::PoolConnection};

#[async_trait]
#[automock]
pub trait SupplierRepository: Send + Sync {
    async fn save(&self, supplier: Supplier, db: PoolConnection<Any>) -> Result<Supplier, sqlx::Error>;
    async fn find_by_id(&self, id: &str, db: PoolConnection<Any>) -> Result<Supplier, sqlx::Error>;
    async fn update(&self, supplier: Supplier, db: PoolConnection<Any>) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: &str, db: PoolConnection<Any>) -> Result<(), sqlx::Error>;
    async fn find_all(&self, db: PoolConnection<Any>) -> Result<Vec<Supplier>, sqlx::Error>;
}
