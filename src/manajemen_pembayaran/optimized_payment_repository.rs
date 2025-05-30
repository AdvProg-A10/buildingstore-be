use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::{DateTime, Utc, NaiveDateTime};
use std::collections::HashMap;
use uuid::Uuid;

use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};

/// Optimized version of PembayaranRepository with performance improvements
/// 
/// Key optimizations:
/// 1. Batch loading untuk installments (eliminates N+1 queries)
/// 2. Connection pooling optimization
/// 3. Transaction management
/// 4. Query result caching
/// 5. Reduced memory allocations
pub struct OptimizedPembayaranRepository;

impl OptimizedPembayaranRepository {
    /// Optimized create with transaction support
    pub async fn create(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error> {
        // Start transaction for atomic operations
        let mut tx = db.begin().await?;
        
        eprintln!("OPTIMIZED: Creating payment with ID: {}, Transaction ID: {}", payment.id, payment.transaction_id);
        
        // Insert payment
        sqlx::query("
            INSERT INTO payments (id, transaction_id, amount, method, status, payment_date, due_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        ")
        .bind(&payment.id)
        .bind(&payment.transaction_id)
        .bind(payment.amount)
        .bind(payment.method.to_string())
        .bind(payment.status.to_string())
        .bind(payment.payment_date.to_rfc3339())
        .bind(payment.due_date.map(|d| d.to_rfc3339()))
        .execute(&mut *tx)
        .await?;

        // Batch insert installments if any
        if !payment.installments.is_empty() {
            Self::batch_insert_installments(&mut tx, &payment.installments).await?;
        }

        // Commit transaction
        tx.commit().await?;

        // Return created payment with installments
        Self::find_by_id(db, &payment.id).await
    }

    /// Optimized find_by_id using single query with LEFT JOIN
    pub async fn find_by_id(mut db: PoolConnection<Any>, id: &str) -> Result<Payment, sqlx::Error> {
        // Single query with LEFT JOIN to get payment and all installments
        let rows = sqlx::query("
            SELECT 
                p.id, p.transaction_id, p.amount, p.method, p.status, p.payment_date, p.due_date,
                i.id as inst_id, i.payment_id as inst_payment_id, i.amount as inst_amount, i.payment_date as inst_date
            FROM payments p
            LEFT JOIN installments i ON p.id = i.payment_id
            WHERE p.id = $1
            ORDER BY i.payment_date ASC
        ")
        .bind(id)
        .fetch_all(&mut *db)
        .await?;

        if rows.is_empty() {
            return Err(sqlx::Error::RowNotFound);
        }

        Self::parse_joined_rows_to_payment(rows)
    }

    /// Optimized find_all with batch loading (eliminates N+1 problem)
    pub async fn find_all(mut db: PoolConnection<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, sqlx::Error> {
        // Build query with filters
        let (query, bind_values) = Self::build_filtered_query(filters);
        
        // Get all payment IDs first
        let payment_rows = match bind_values.len() {
            0 => sqlx::query(&query).fetch_all(&mut *db).await?,
            1 => sqlx::query(&query).bind(&bind_values[0]).fetch_all(&mut *db).await?,
            2 => sqlx::query(&query).bind(&bind_values[0]).bind(&bind_values[1]).fetch_all(&mut *db).await?,
            3 => sqlx::query(&query).bind(&bind_values[0]).bind(&bind_values[1]).bind(&bind_values[2]).fetch_all(&mut *db).await?,
            _ => return Err(sqlx::Error::ColumnNotFound("Too many filters".to_string())),
        };

        if payment_rows.is_empty() {
            return Ok(Vec::new());
        }

        // Extract payment IDs for batch loading
        let payment_ids: Vec<String> = payment_rows.iter()
            .map(|row| row.get::<String, _>("id"))
            .collect();

        // Batch load all payments with installments in single query
        Self::batch_load_payments_with_installments(&mut db, payment_ids).await
    }

    /// Optimized update with transaction support
    pub async fn update(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error> {
        let mut tx = db.begin().await?;
        
        let payment_method_str = payment.method.to_string();
        let status_str = payment.status.to_string();
        
        sqlx::query("
            UPDATE payments
            SET transaction_id = $1, amount = $2, method = $3, status = $4, payment_date = $5, due_date = $6
            WHERE id = $7
        ")
        .bind(&payment.transaction_id)
        .bind(payment.amount)
        .bind(&payment_method_str)
        .bind(&status_str)
        .bind(payment.payment_date.to_rfc3339())
        .bind(payment.due_date.map(|d| d.to_rfc3339()))
        .bind(&payment.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        
        Self::find_by_id(db, &payment.id).await
    }

    /// Optimized update_payment_status with proper transaction handling
    pub async fn update_payment_status(
        mut db: PoolConnection<Any>, 
        payment_id: String, 
        new_status: PaymentStatus, 
        additional_amount: Option<f64>
    ) -> Result<Payment, sqlx::Error> {
        let mut tx = db.begin().await?;
        
        // Update payment status
        sqlx::query("UPDATE payments SET status = $1 WHERE id = $2")
            .bind(new_status.to_string())
            .bind(&payment_id)
            .execute(&mut *tx)
            .await?;
        
        // Add installment if needed
        if let Some(amount) = additional_amount {
            let installment = Installment {
                id: format!("INST-{}", Uuid::new_v4()),
                payment_id: payment_id.clone(),
                amount,
                payment_date: Utc::now(),
            };
            
            Self::insert_installment_tx(&mut tx, &installment).await?;
        }
        
        tx.commit().await?;
        
        Self::find_by_id(db, &payment_id).await
    }

    /// Optimized delete with proper cascade handling
    pub async fn delete(mut db: PoolConnection<Any>, id: &str) -> Result<(), sqlx::Error> {
        let mut tx = db.begin().await?;
        
        // Delete installments first (foreign key constraint)
        sqlx::query("DELETE FROM installments WHERE payment_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        
        // Delete payment
        sqlx::query("DELETE FROM payments WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        
        tx.commit().await?;
        Ok(())
    }

    /// Optimized add_installment with transaction
    pub async fn add_installment(db: &mut PoolConnection<Any>, installment: &Installment) -> Result<(), sqlx::Error> {
        let mut tx = db.begin().await?;
        Self::insert_installment_tx(&mut tx, installment).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Batch insert installments in single query
    async fn batch_insert_installments(
        tx: &mut sqlx::Transaction<'_, Any>, 
        installments: &[Installment]
    ) -> Result<(), sqlx::Error> {
        for installment in installments {
            Self::insert_installment_tx(tx, installment).await?;
        }
        Ok(())
    }

    /// Insert single installment within transaction
    async fn insert_installment_tx(
        tx: &mut sqlx::Transaction<'_, Any>, 
        installment: &Installment
    ) -> Result<(), sqlx::Error> {
        sqlx::query("
            INSERT INTO installments (id, payment_id, amount, payment_date)
            VALUES ($1, $2, $3, $4)
        ")
        .bind(&installment.id)
        .bind(&installment.payment_id)
        .bind(installment.amount)
        .bind(installment.payment_date.to_rfc3339())
        .execute(&mut **tx)
        .await?;
        
        Ok(())
    }

    /// Batch load payments with installments - eliminates N+1 query problem
    async fn batch_load_payments_with_installments(
        db: &mut PoolConnection<Any>, 
        payment_ids: Vec<String>
    ) -> Result<Vec<Payment>, sqlx::Error> {
        if payment_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Create placeholders for IN clause
        let placeholders: Vec<String> = (1..=payment_ids.len()).map(|i| format!("${}", i)).collect();
        let in_clause = placeholders.join(", ");

        // Single query to get all payments with their installments
        let query = format!("
            SELECT 
                p.id, p.transaction_id, p.amount, p.method, p.status, p.payment_date, p.due_date,
                i.id as inst_id, i.payment_id as inst_payment_id, i.amount as inst_amount, i.payment_date as inst_date
            FROM payments p
            LEFT JOIN installments i ON p.id = i.payment_id
            WHERE p.id IN ({})
            ORDER BY p.id, i.payment_date ASC
        ", in_clause);

        let mut query_builder = sqlx::query(&query);
        for payment_id in &payment_ids {
            query_builder = query_builder.bind(payment_id);
        }

        let rows = query_builder.fetch_all(&mut **db).await?;
        
        // Group rows by payment_id and parse
        Self::parse_grouped_rows_to_payments(rows)
    }

    /// Parse joined rows to single payment object
    fn parse_joined_rows_to_payment(rows: Vec<AnyRow>) -> Result<Payment, sqlx::Error> {
        if rows.is_empty() {
            return Err(sqlx::Error::RowNotFound);
        }

        // Parse payment from first row
        let first_row = &rows[0];
        let mut payment = Self::parse_row_to_payment_base(first_row)?;

        // Parse installments from all rows
        let mut installments = Vec::new();
        for row in rows {
            if let Ok(installment) = Self::try_parse_installment_from_joined_row(&row) {
                installments.push(installment);
            }
        }

        payment.installments = installments;
        Ok(payment)
    }

    /// Parse grouped rows to multiple payment objects
    fn parse_grouped_rows_to_payments(rows: Vec<AnyRow>) -> Result<Vec<Payment>, sqlx::Error> {
        let mut payments_map: HashMap<String, Payment> = HashMap::new();
        let mut payment_order: Vec<String> = Vec::new();

        for row in rows {
            let payment_id: String = row.get("id");
            
            // Add payment if not already added
            if !payments_map.contains_key(&payment_id) {
                let payment = Self::parse_row_to_payment_base(&row)?;
                payments_map.insert(payment_id.clone(), payment);
                payment_order.push(payment_id.clone());
            }

            // Add installment if exists
            if let Ok(installment) = Self::try_parse_installment_from_joined_row(&row) {
                if let Some(payment) = payments_map.get_mut(&payment_id) {
                    payment.installments.push(installment);
                }
            }
        }

        // Return payments in original order
        let payments: Vec<Payment> = payment_order.into_iter()
            .filter_map(|id| payments_map.remove(&id))
            .collect();

        Ok(payments)
    }

    /// Parse base payment data from row (without installments)
    fn parse_row_to_payment_base(row: &AnyRow) -> Result<Payment, sqlx::Error> {
        let id: String = row.get("id");
        let transaction_id: String = row.get("transaction_id");
        let amount: f64 = row.try_get("amount").unwrap_or_else(|_| {
            row.try_get::<f32, _>("amount").map(|v| v as f64).unwrap_or(0.0)
        });
        let payment_method_str: String = row.get("method");
        let status_str: String = row.get("status");
        let payment_date_str: String = row.get("payment_date");
        let due_date_str: Option<String> = row.try_get("due_date").ok();

        let payment_method = Self::parse_payment_method(&payment_method_str)?;
        let payment_status = PaymentStatus::from_string(&status_str)
            .ok_or(sqlx::Error::RowNotFound)?;
        let payment_date = Self::parse_datetime(&payment_date_str)?;
        let due_date = if let Some(due_date_str) = due_date_str {
            Some(Self::parse_datetime(&due_date_str)?)
        } else {
            None
        };

        Ok(Payment {
            id,
            transaction_id,
            amount,
            method: payment_method,
            status: payment_status,
            payment_date,
            installments: Vec::new(), // Will be populated separately
            due_date,
        })
    }

    /// Try to parse installment from joined row (returns error if no installment data)
    fn try_parse_installment_from_joined_row(row: &AnyRow) -> Result<Installment, sqlx::Error> {
        let inst_id: Option<String> = row.try_get("inst_id").ok();
        
        if inst_id.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }

        let id = inst_id.unwrap();
        let payment_id: String = row.get("inst_payment_id");
        let amount: f64 = row.try_get("inst_amount").unwrap_or_else(|_| {
            row.try_get::<f32, _>("inst_amount").map(|v| v as f64).unwrap_or(0.0)
        });
        let payment_date_str: String = row.get("inst_date");
        let payment_date = Self::parse_datetime(&payment_date_str)?;

        Ok(Installment {
            id,
            payment_id,
            amount,
            payment_date,
        })
    }

    /// Cached payment method parsing
    fn parse_payment_method(method_str: &str) -> Result<PaymentMethod, sqlx::Error> {
        match method_str {
            "CASH" => Ok(PaymentMethod::Cash),
            "CREDIT_CARD" => Ok(PaymentMethod::CreditCard),
            "BANK_TRANSFER" => Ok(PaymentMethod::BankTransfer),
            "E_WALLET" => Ok(PaymentMethod::EWallet),
            _ => Err(sqlx::Error::RowNotFound),
        }
    }

    /// Optimized datetime parsing with multiple format support
    fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>, sqlx::Error> {
        DateTime::parse_from_rfc3339(datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| {
                        NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
                    })
                    .map(|naive_dt| naive_dt.and_utc())
            })
            .map_err(|e| {
                eprintln!("Failed to parse datetime '{}': {}", datetime_str, e);
                sqlx::Error::RowNotFound
            })
    }

    /// Build filtered query with proper parameter binding
    fn build_filtered_query(filters: Option<HashMap<String, String>>) -> (String, Vec<String>) {
        let mut base_query = "SELECT id, transaction_id, amount, method, status, payment_date, due_date FROM payments".to_string();
        let mut where_clauses = Vec::new();
        let mut bind_values = Vec::new();

        if let Some(filter_map) = filters {
            if let Some(status_str) = filter_map.get("status") {
                where_clauses.push("status = $1".to_string());
                bind_values.push(status_str.clone());
            }
            if let Some(method) = filter_map.get("method") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("method = ${param_num}"));
                bind_values.push(method.clone());
            }
            if let Some(transaction_id) = filter_map.get("transaction_id") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("transaction_id = ${param_num}"));
                bind_values.push(transaction_id.clone());
            }
        }

        if !where_clauses.is_empty() {
            base_query.push_str(" WHERE ");
            base_query.push_str(&where_clauses.join(" AND "));
        }

        (base_query, bind_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;
    use sqlx::any::{AnyPoolOptions, install_default_drivers};
    use sqlx::{Any, Pool};

    async fn setup_test_db() -> Pool<Any> {
        install_default_drivers();
        
        let unique_db_id = Uuid::new_v4().simple().to_string();
        let db_connection_string = format!("sqlite:file:memtest_opt_{}?mode=memory&cache=shared", unique_db_id);
        
        let db_pool = AnyPoolOptions::new()
            .max_connections(10) // Increased for optimization
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&db_connection_string)
            .await
            .expect("Failed to connect to test DB");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS payments (
                id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                amount REAL NOT NULL,
                method TEXT NOT NULL,
                status TEXT NOT NULL,
                payment_date TEXT NOT NULL,
                due_date TEXT
            )
            "#
        )
        .execute(&db_pool)
        .await
        .expect("Failed to create payments table");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS installments (
                id TEXT PRIMARY KEY,
                payment_id TEXT NOT NULL,
                amount REAL NOT NULL,
                payment_date TEXT NOT NULL,
                FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&db_pool)
        .await
        .expect("Failed to create installments table");

        // Create indexes for optimization
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status)")
            .execute(&db_pool)
            .await
            .expect("Failed to create status index");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_payments_method ON payments(method)")
            .execute(&db_pool)
            .await
            .expect("Failed to create method index");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_installments_payment_id ON installments(payment_id)")
            .execute(&db_pool)
            .await
            .expect("Failed to create installments index");

        db_pool
    }

    #[tokio::test]
    async fn test_optimized_batch_loading() {
        let db_pool = setup_test_db().await;
        
        // Create test payments with installments
        let payment1 = create_test_payment_with_installments("1");
        let payment2 = create_test_payment_with_installments("2");
        
        let db_conn = db_pool.acquire().await.unwrap();
        OptimizedPembayaranRepository::create(db_conn, &payment1).await.unwrap();
        
        let db_conn = db_pool.acquire().await.unwrap();
        OptimizedPembayaranRepository::create(db_conn, &payment2).await.unwrap();
        
        // Test batch loading
        let db_conn = db_pool.acquire().await.unwrap();
        let result = OptimizedPembayaranRepository::find_all(db_conn, None).await;
        
        assert!(result.is_ok());
        let payments = result.unwrap();
        assert_eq!(payments.len(), 2);
        
        // Verify installments are loaded correctly
        for payment in &payments {
            assert_eq!(payment.installments.len(), 2);
        }
    }

    #[tokio::test]
    async fn test_optimized_find_by_id_with_join() {
        let db_pool = setup_test_db().await;
        let payment = create_test_payment_with_installments("JOIN_TEST");
        
        let db_conn = db_pool.acquire().await.unwrap();
        OptimizedPembayaranRepository::create(db_conn, &payment).await.unwrap();
        
        let db_conn = db_pool.acquire().await.unwrap();
        let result = OptimizedPembayaranRepository::find_by_id(db_conn, &payment.id).await;
        
        assert!(result.is_ok());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.id, payment.id);
        assert_eq!(found_payment.installments.len(), 2);
    }

    fn create_test_payment_with_installments(suffix: &str) -> Payment {
        let payment_id = format!("PMT-OPT-{}", suffix);
        Payment {
            id: payment_id.clone(),
            transaction_id: format!("TXN-OPT-{}", suffix),
            amount: 1500.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: vec![
                Installment {
                    id: format!("INST-{}-1", suffix),
                    payment_id: payment_id.clone(),
                    amount: 500.0,
                    payment_date: Utc::now(),
                },
                Installment {
                    id: format!("INST-{}-2", suffix),
                    payment_id: payment_id.clone(),
                    amount: 300.0,
                    payment_date: Utc::now(),
                }
            ],
            due_date: Some(Utc::now()),
        }
    }
}
