use sqlx::{Any, pool::PoolConnection, any::AnyRow, Row};
use async_trait::async_trait;
use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;

pub struct SupplierTransactionRepositoryImpl;

impl SupplierTransactionRepositoryImpl {
    pub fn new() -> Self {
        Self
    }

    fn parse_row_to_transaction(row: AnyRow) -> Result<SupplierTransaction, sqlx::Error> {
        let id: String = row.get("id");
        let supplier_id: String = row.get("supplier_id");
        let supplier_name: String = row.get("supplier_name");
        let jenis_barang: String = row.get("jenis_barang");
        let jumlah_barang: i32 = row.get("jumlah_barang");
        let pengiriman_info: String = row.get("pengiriman_info");
        let tanggal_transaksi: String = row.get("tanggal_transaksi");

        Ok(SupplierTransaction {
            id,
            supplier_id,
            supplier_name,
            jenis_barang,
            jumlah_barang,
            pengiriman_info,
            tanggal_transaksi,
        })
    }
}

#[async_trait]
impl SupplierTransactionRepository for SupplierTransactionRepositoryImpl {
    async fn save(&self, transaction: SupplierTransaction, mut db: PoolConnection<Any>) -> Result<SupplierTransaction, sqlx::Error> {
        let query = "
            INSERT INTO supplier_transactions (id, supplier_id, supplier_name, jenis_barang, jumlah_barang, pengiriman_info, tanggal_transaksi)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        ";

        sqlx::query(query)
            .bind(&transaction.id)
            .bind(&transaction.supplier_id)
            .bind(&transaction.supplier_name)
            .bind(&transaction.jenis_barang)
            .bind(transaction.jumlah_barang)
            .bind(&transaction.pengiriman_info)
            .bind(&transaction.tanggal_transaksi)
            .execute(&mut *db)
            .await?;

        Ok(transaction)
    }

    async fn find_by_id(&self, id: &str, mut db: PoolConnection<Any>) -> Result<SupplierTransaction, sqlx::Error> {
        let query = "SELECT * FROM supplier_transactions WHERE id = $1";

        let row = sqlx::query(query)
            .bind(id)
            .fetch_one(&mut *db)
            .await?;

        Self::parse_row_to_transaction(row)
    }

    async fn find_by_supplier_id(&self, supplier_id: &str, mut db: PoolConnection<Any>) -> Result<Vec<SupplierTransaction>, sqlx::Error> {
        let query = "SELECT * FROM supplier_transactions WHERE supplier_id = $1";

        let rows = sqlx::query(query)
            .bind(supplier_id)
            .fetch_all(&mut *db)
            .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Self::parse_row_to_transaction(row)?);
        }

        Ok(transactions)
    }

    async fn find_all(&self, mut db: PoolConnection<Any>) -> Result<Vec<SupplierTransaction>, sqlx::Error> {
        let query = "SELECT * FROM supplier_transactions";
        let rows = sqlx::query(query)
            .fetch_all(&mut *db)
            .await?;

        let mut suppliers = Vec::new();
        for row in rows {
            suppliers.push(Self::parse_row_to_transaction(row)?);
        }
        Ok(suppliers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}, Pool}; 
    use chrono::Utc;
    use uuid::Uuid;
    use crate::manajemen_supplier::{
        model::supplier::Supplier, 
        repository::supplier_repository_impl::SupplierRepositoryImpl,
        repository::supplier_repository::SupplierRepository,
    };


    async fn setup_repository() -> (SupplierTransactionRepositoryImpl, SupplierRepositoryImpl, Pool<Any>) {
        install_default_drivers();
        let db_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test DB");

        sqlx::migrate!("migrations/test")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations. Ensure 'migrations/test' directory exists and migrations are valid for SQLite.");

        (SupplierTransactionRepositoryImpl::new(), SupplierRepositoryImpl::new(), db_pool)
    }

    fn create_supplier() -> Supplier {
        Supplier {
            id: format!("SUP-{}", Uuid::new_v4()),
            name: "PT. Test Supplier".to_string(),
            jenis_barang: "Test Goods".to_string(),
            jumlah_barang: 100,
            resi: "RESI-TEST-001".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    fn create_transaction(supplier: &Supplier) -> SupplierTransaction {
        SupplierTransaction {
            id: format!("TRX-{}", Uuid::new_v4()),
            supplier_id: supplier.id.clone(),
            supplier_name: supplier.name.clone(),
            jenis_barang: supplier.jenis_barang.clone(),
            jumlah_barang: supplier.jumlah_barang, 
            pengiriman_info: format!("Info for {}", supplier.resi.clone()),
            tanggal_transaksi: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_save_supplier_transaction() { 
        let (transaction_repo, supplier_repo, db_pool) = setup_repository().await;
        let supplier = create_supplier();
        
        let db_conn_supplier = db_pool.acquire().await.unwrap();
        supplier_repo.save(supplier.clone(), db_conn_supplier).await.expect("Saving supplier failed");
        
        let transaksi = create_transaction(&supplier);
        let db_conn_trans = db_pool.acquire().await.unwrap();
        let result = transaction_repo.save(transaksi.clone(), db_conn_trans).await;
        
        assert!(result.is_ok(), "Saving transaction failed: {:?}", result.err());
        let saved = result.unwrap();
        assert_eq!(saved.id, transaksi.id);
        assert_eq!(saved.supplier_id, supplier.id);
        assert_eq!(saved.supplier_name, supplier.name);
    }

    #[tokio::test]
    async fn test_find_supplier_transaction_by_id() {
        let (transaction_repo, supplier_repo, db_pool) = setup_repository().await;
        let supplier = create_supplier();
        let transaksi = create_transaction(&supplier);

        let db_conn_supplier = db_pool.acquire().await.unwrap();
        supplier_repo.save(supplier.clone(), db_conn_supplier).await.unwrap();

        let db_conn_trans_save = db_pool.acquire().await.unwrap();
        transaction_repo.save(transaksi.clone(), db_conn_trans_save).await.unwrap();
        
        let db_conn_trans_find = db_pool.acquire().await.unwrap();
        let found_result = transaction_repo.find_by_id(&transaksi.id, db_conn_trans_find).await;
        assert!(found_result.is_ok(), "Find by ID failed: {:?}", found_result.err());
        assert_eq!(found_result.unwrap().id, transaksi.id);
    }

    #[tokio::test]
    async fn test_find_nonexistent_transaction_by_id() { 
        let (transaction_repo, _supplier_repo, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let result = transaction_repo.find_by_id("TRX-NON-EXISTENT", db_conn).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {}, 
            e => panic!("Expected RowNotFound error, but got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_find_supplier_transactions_by_supplier_id() { 
        let (transaction_repo, supplier_repo, db_pool) = setup_repository().await;
        let supplier = create_supplier();
        let transaksi1 = create_transaction(&supplier);
        let mut transaksi2 = create_transaction(&supplier); 
        transaksi2.id = format!("TRX-{}", Uuid::new_v4()); 
        let db_conn_supplier = db_pool.acquire().await.unwrap();
        supplier_repo.save(supplier.clone(), db_conn_supplier).await.unwrap();

        let db_conn_t1 = db_pool.acquire().await.unwrap();
        transaction_repo.save(transaksi1.clone(), db_conn_t1).await.unwrap();
        
        let db_conn_t2 = db_pool.acquire().await.unwrap();
        transaction_repo.save(transaksi2.clone(), db_conn_t2).await.unwrap();

        let db_conn_find = db_pool.acquire().await.unwrap();
        let results = transaction_repo.find_by_supplier_id(&supplier.id, db_conn_find).await;
        assert!(results.is_ok(), "Find by supplier ID failed: {:?}", results.err());
        let transactions = results.unwrap();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.iter().any(|t| t.id == transaksi1.id));
        assert!(transactions.iter().any(|t| t.id == transaksi2.id));
        assert!(transactions.iter().all(|t| t.supplier_id == supplier.id));
    }

    #[tokio::test]
    async fn test_find_transactions_for_nonexistent_supplier_id() {
        let (transaction_repo, _supplier_repo, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let results = transaction_repo.find_by_supplier_id("SUP-NON-EXISTENT", db_conn).await;
        
        assert!(results.is_ok(), "Find by non-existent supplier ID failed: {:?}", results.err());
        let transactions = results.unwrap();
        assert!(transactions.is_empty(), "Expected no transactions for a non-existent supplier ID");
    }

    #[tokio::test]
    async fn test_find_all_transactions_empty() {
        let (transaction_repo, _supplier_repo, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let result = transaction_repo.find_all(db_conn).await;
        
        assert!(result.is_ok(), "find_all (empty) failed: {:?}", result.err());
        let transactions = result.unwrap();
        assert!(transactions.is_empty(), "Expected empty vector for transactions, but found {}", transactions.len());
    }

    #[tokio::test]
    async fn test_find_all_transactions_multiple() {
        let (transaction_repo, supplier_repo, db_pool) = setup_repository().await;
        
        let supplier1 = create_supplier();
        let mut supplier2 = create_supplier(); 
        supplier2.id = format!("SUP-{}", Uuid::new_v4());
        
        let db_conn_s1 = db_pool.acquire().await.unwrap();
        supplier_repo.save(supplier1.clone(), db_conn_s1).await.unwrap();
        
        let db_conn_s2 = db_pool.acquire().await.unwrap();
        supplier_repo.save(supplier2.clone(), db_conn_s2).await.unwrap();

        let transaction1_s1 = create_transaction(&supplier1);
        let transaction2_s2 = create_transaction(&supplier2); 
        let mut transaction3_s1 = create_transaction(&supplier1);
        transaction3_s1.id = format!("TRX-{}", Uuid::new_v4());


        let db_conn_t1s1 = db_pool.acquire().await.unwrap();
        transaction_repo.save(transaction1_s1.clone(), db_conn_t1s1).await.unwrap();
        
        let db_conn_t2s2 = db_pool.acquire().await.unwrap();
        transaction_repo.save(transaction2_s2.clone(), db_conn_t2s2).await.unwrap();

        let db_conn_t3s1 = db_pool.acquire().await.unwrap();
        transaction_repo.save(transaction3_s1.clone(), db_conn_t3s1).await.unwrap();


        let db_conn_find_all = db_pool.acquire().await.unwrap();
        let result = transaction_repo.find_all(db_conn_find_all).await;
        
        assert!(result.is_ok(), "find_all (multiple) failed: {:?}", result.err());
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 3, "Expected 3 transactions in total, found {}", transactions.len());
        
        let ids: Vec<String> = transactions.iter().map(|t| t.id.clone()).collect();
        assert!(ids.contains(&transaction1_s1.id));
        assert!(ids.contains(&transaction2_s2.id));
        assert!(ids.contains(&transaction3_s1.id));
    }
}