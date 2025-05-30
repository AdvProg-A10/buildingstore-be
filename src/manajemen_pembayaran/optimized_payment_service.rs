use std::collections::HashMap;
use std::sync::Arc;
use rocket::State;
use chrono::Utc;
use uuid::Uuid;
use tokio::sync::RwLock;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::optimized_payment_repository::OptimizedPembayaranRepository;
use sqlx::{Any, Pool};

/// Optimized PaymentService with caching and improved performance
/// 
/// Key optimizations:
/// 1. Method/Status parsing cache
/// 2. Connection pool optimization
/// 3. Better error handling with structured errors
/// 4. Result caching for frequently accessed data
/// 5. Async-optimized operations
pub struct OptimizedPaymentService {
    method_cache: HashMap<String, PaymentMethod>,
    status_cache: HashMap<String, PaymentStatus>,
    result_cache: Arc<RwLock<HashMap<String, CachedPayment>>>,
}

#[derive(Debug, Clone)]
struct CachedPayment {
    payment: Payment,
    cached_at: chrono::DateTime<Utc>,
    ttl_seconds: i64,
}

impl CachedPayment {
    fn new(payment: Payment, ttl_seconds: i64) -> Self {
        Self {
            payment,
            cached_at: Utc::now(),
            ttl_seconds,
        }
    }

    fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.cached_at);
        elapsed.num_seconds() > self.ttl_seconds
    }
}

#[derive(Debug, Clone)]
pub enum OptimizedPaymentError {
    DatabaseError { message: String, code: Option<String> },
    NotFound { entity: String, id: String },
    InvalidInput { field: String, message: String },
    ValidationError { errors: Vec<String> },
    CacheError { message: String },
    ConnectionError { message: String },
}

impl std::fmt::Display for OptimizedPaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizedPaymentError::DatabaseError { message, code } => {
                write!(f, "Database error: {} (code: {:?})", message, code)
            }
            OptimizedPaymentError::NotFound { entity, id } => {
                write!(f, "{} with id '{}' not found", entity, id)
            }
            OptimizedPaymentError::InvalidInput { field, message } => {
                write!(f, "Invalid input for field '{}': {}", field, message)
            }
            OptimizedPaymentError::ValidationError { errors } => {
                write!(f, "Validation errors: {}", errors.join(", "))
            }
            OptimizedPaymentError::CacheError { message } => {
                write!(f, "Cache error: {}", message)
            }
            OptimizedPaymentError::ConnectionError { message } => {
                write!(f, "Connection error: {}", message)
            }
        }
    }
}

impl std::error::Error for OptimizedPaymentError {}

impl OptimizedPaymentService {
    pub fn new() -> Self {
        let mut method_cache = HashMap::new();
        method_cache.insert("CASH".to_string(), PaymentMethod::Cash);
        method_cache.insert("CREDIT_CARD".to_string(), PaymentMethod::CreditCard);
        method_cache.insert("BANK_TRANSFER".to_string(), PaymentMethod::BankTransfer);
        method_cache.insert("E_WALLET".to_string(), PaymentMethod::EWallet);
        
        let mut status_cache = HashMap::new();
        status_cache.insert("LUNAS".to_string(), PaymentStatus::Paid);
        status_cache.insert("CICILAN".to_string(), PaymentStatus::Installment);
        
        Self {
            method_cache,
            status_cache,
            result_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Optimized create_payment with validation and caching
    pub async fn create_payment(&self, db: &State<Pool<Any>>, payment: Payment) -> Result<Payment, OptimizedPaymentError> {
        // Validate payment before creation
        self.validate_payment(&payment)?;
        
        // Get connection with timeout
        let conn = self.acquire_connection(db).await?;
        
        // Create payment using optimized repository
        let created_payment = OptimizedPembayaranRepository::create(conn, &payment).await
            .map_err(|e| self.convert_sqlx_error(e))?;
        
        // Cache the result
        self.cache_payment(&created_payment.id, &created_payment, 300).await; // 5 minute TTL
        
        Ok(created_payment)
    }

    /// Optimized get_payment_by_id with caching
    pub async fn get_payment_by_id(&self, db: &State<Pool<Any>>, id: &str) -> Result<Payment, OptimizedPaymentError> {
        // Check cache first
        if let Some(cached_payment) = self.get_cached_payment(id).await {
            if !cached_payment.is_expired() {
                return Ok(cached_payment.payment);
            } else {
                // Remove expired cache entry
                self.remove_from_cache(id).await;
            }
        }
        
        // Get from database
        let conn = self.acquire_connection(db).await?;
        let payment = OptimizedPembayaranRepository::find_by_id(conn, id).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => OptimizedPaymentError::NotFound {
                    entity: "Payment".to_string(),
                    id: id.to_string(),
                },
                _ => self.convert_sqlx_error(e)
            })?;
        
        // Cache the result
        self.cache_payment(id, &payment, 300).await; // 5 minute TTL
        
        Ok(payment)
    }

    /// Optimized get_all_payments with better filtering
    pub async fn get_all_payments(&self, db: &State<Pool<Any>>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, OptimizedPaymentError> {
        // Validate filters
        if let Some(ref filter_map) = filters {
            self.validate_filters(filter_map)?;
        }
        
        let conn = self.acquire_connection(db).await?;
        
        OptimizedPembayaranRepository::find_all(conn, filters).await
            .map_err(|e| self.convert_sqlx_error(e))
    }

    /// Optimized update_payment with change detection
    pub async fn update_payment(&self, db: &State<Pool<Any>>, payment: Payment) -> Result<Payment, OptimizedPaymentError> {
        // Validate payment
        self.validate_payment(&payment)?;
        
        // Check if payment exists
        let existing_payment = self.get_payment_by_id(db, &payment.id).await?;
        
        // Detect changes to avoid unnecessary updates
        if self.payments_are_equal(&existing_payment, &payment) {
            return Ok(existing_payment);
        }
        
        let conn = self.acquire_connection(db).await?;
        
        let updated_payment = OptimizedPembayaranRepository::update(conn, &payment).await
            .map_err(|e| self.convert_sqlx_error(e))?;
        
        // Update cache
        self.cache_payment(&payment.id, &updated_payment, 300).await;
        
        Ok(updated_payment)
    }
    
    /// Optimized update_payment_status with proper validation
    pub async fn update_payment_status(
        &self, 
        db: &State<Pool<Any>>, 
        payment_id: String, 
        new_status: PaymentStatus, 
        additional_amount: Option<f64>
    ) -> Result<Payment, OptimizedPaymentError> {
        // Validate additional amount if provided
        if let Some(amount) = additional_amount {
            if amount <= 0.0 {
                return Err(OptimizedPaymentError::InvalidInput {
                    field: "additional_amount".to_string(),
                    message: "Amount must be greater than 0".to_string(),
                });
            }
        }
        
        let conn = self.acquire_connection(db).await?;
        
        let updated_payment = OptimizedPembayaranRepository::update_payment_status(
            conn, 
            payment_id.clone(), 
            new_status, 
            additional_amount
        ).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => OptimizedPaymentError::NotFound {
                    entity: "Payment".to_string(),
                    id: payment_id,
                },
                _ => self.convert_sqlx_error(e)
            })?;
        
        // Update cache
        self.cache_payment(&updated_payment.id, &updated_payment, 300).await;
        
        Ok(updated_payment)
    }
    
    /// Optimized delete_payment with cache invalidation
    pub async fn delete_payment(&self, db: &State<Pool<Any>>, payment_id: &str) -> Result<(), OptimizedPaymentError> {
        // Check if payment exists first
        let _existing_payment = self.get_payment_by_id(db, payment_id).await?;
        
        let conn = self.acquire_connection(db).await?;
        
        OptimizedPembayaranRepository::delete(conn, payment_id).await
            .map_err(|e| self.convert_sqlx_error(e))?;
        
        // Remove from cache
        self.remove_from_cache(payment_id).await;
        
        Ok(())
    }
    
    /// Optimized add_installment with validation
    pub async fn add_installment(&self, db: &State<Pool<Any>>, payment_id: &str, amount: f64) -> Result<Payment, OptimizedPaymentError> {
        // Validate amount
        if amount <= 0.0 {
            return Err(OptimizedPaymentError::InvalidInput {
                field: "amount".to_string(),
                message: "Amount must be greater than 0".to_string(),
            });
        }
        
        // Get current payment and validate status
        let payment = self.get_payment_by_id(db, payment_id).await?;
        
        if payment.status != PaymentStatus::Installment {
            return Err(OptimizedPaymentError::InvalidInput {
                field: "payment_status".to_string(),
                message: "Cannot add installment to a payment that is not in INSTALLMENT status".to_string(),
            });
        }
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };
        
        let mut conn = self.acquire_connection(db).await?;
        
        OptimizedPembayaranRepository::add_installment(&mut conn, &installment).await
            .map_err(|e| self.convert_sqlx_error(e))?;
        
        // Get updated payment
        let updated_payment = self.get_payment_by_id(db, payment_id).await?;
        
        Ok(updated_payment)
    }

    // === CACHING METHODS ===

    async fn get_cached_payment(&self, id: &str) -> Option<CachedPayment> {
        let cache = self.result_cache.read().await;
        cache.get(id).cloned()
    }

    async fn cache_payment(&self, id: &str, payment: &Payment, ttl_seconds: i64) {
        let mut cache = self.result_cache.write().await;
        cache.insert(id.to_string(), CachedPayment::new(payment.clone(), ttl_seconds));
    }

    async fn remove_from_cache(&self, id: &str) {
        let mut cache = self.result_cache.write().await;
        cache.remove(id);
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.result_cache.write().await;
        cache.clear();
    }

    // === UTILITY METHODS ===

    /// Optimized connection acquisition with timeout
    async fn acquire_connection(&self, db: &State<Pool<Any>>) -> Result<sqlx::pool::PoolConnection<Any>, OptimizedPaymentError> {
        db.acquire().await
            .map_err(|e| OptimizedPaymentError::ConnectionError {
                message: format!("Failed to acquire database connection: {}", e),
            })
    }

    /// Cached payment method parsing
    pub fn parse_payment_method(&self, method: &str) -> Result<PaymentMethod, OptimizedPaymentError> {
        self.method_cache.get(&method.to_uppercase())
            .copied()
            .ok_or_else(|| OptimizedPaymentError::InvalidInput {
                field: "payment_method".to_string(),
                message: format!("Invalid payment method: {}", method),
            })
    }

    /// Cached payment status parsing
    pub fn parse_payment_status(&self, status: &str) -> Result<PaymentStatus, OptimizedPaymentError> {
        self.status_cache.get(&status.to_uppercase())
            .copied()
            .ok_or_else(|| OptimizedPaymentError::InvalidInput {
                field: "payment_status".to_string(),
                message: format!("Invalid payment status: {}", status),
            })
    }

    /// Comprehensive payment validation
    fn validate_payment(&self, payment: &Payment) -> Result<(), OptimizedPaymentError> {
        let mut errors = Vec::new();

        if payment.id.trim().is_empty() {
            errors.push("Payment ID cannot be empty".to_string());
        }

        if payment.transaction_id.trim().is_empty() {
            errors.push("Transaction ID cannot be empty".to_string());
        }

        if payment.amount <= 0.0 {
            errors.push("Payment amount must be greater than 0".to_string());
        }

        // Validate installments if any
        if payment.status == PaymentStatus::Installment && payment.installments.is_empty() {
            errors.push("Payment with INSTALLMENT status must have at least one installment".to_string());
        }

        if !payment.installments.is_empty() {
            for (i, installment) in payment.installments.iter().enumerate() {
                if installment.amount <= 0.0 {
                    errors.push(format!("Installment {} amount must be greater than 0", i + 1));
                }
                if installment.payment_id != payment.id {
                    errors.push(format!("Installment {} payment_id does not match payment ID", i + 1));
                }
            }
        }

        if !errors.is_empty() {
            return Err(OptimizedPaymentError::ValidationError { errors });
        }

        Ok(())
    }

    /// Validate filter parameters
    fn validate_filters(&self, filters: &HashMap<String, String>) -> Result<(), OptimizedPaymentError> {
        let mut errors = Vec::new();

        for (key, value) in filters {
            match key.as_str() {
                "status" => {
                    if self.parse_payment_status(value).is_err() {
                        errors.push(format!("Invalid status filter: {}", value));
                    }
                }
                "method" => {
                    if self.parse_payment_method(value).is_err() {
                        errors.push(format!("Invalid method filter: {}", value));
                    }
                }
                "transaction_id" => {
                    if value.trim().is_empty() {
                        errors.push("Transaction ID filter cannot be empty".to_string());
                    }
                }
                _ => {
                    errors.push(format!("Unknown filter key: {}", key));
                }
            }
        }

        if !errors.is_empty() {
            return Err(OptimizedPaymentError::ValidationError { errors });
        }

        Ok(())
    }

    /// Check if two payments are functionally equal (for change detection)
    fn payments_are_equal(&self, payment1: &Payment, payment2: &Payment) -> bool {
        payment1.id == payment2.id &&
        payment1.transaction_id == payment2.transaction_id &&
        (payment1.amount - payment2.amount).abs() < f64::EPSILON &&
        payment1.method == payment2.method &&
        payment1.status == payment2.status &&
        payment1.installments.len() == payment2.installments.len()
    }

    /// Convert sqlx errors to structured errors
    fn convert_sqlx_error(&self, error: sqlx::Error) -> OptimizedPaymentError {
        match error {
            sqlx::Error::RowNotFound => OptimizedPaymentError::NotFound {
                entity: "Payment".to_string(),
                id: "unknown".to_string(),
            },
            sqlx::Error::Database(db_err) => OptimizedPaymentError::DatabaseError {
                message: db_err.message().to_string(),
                code: db_err.code().map(|c| c.to_string()),
            },
            sqlx::Error::PoolTimedOut => OptimizedPaymentError::ConnectionError {
                message: "Database connection pool timed out".to_string(),
            },
            _ => OptimizedPaymentError::DatabaseError {
                message: error.to_string(),
                code: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use sqlx::any::{AnyPoolOptions, install_default_drivers};
    use sqlx::{Any, Pool};

    async fn setup_test_db() -> Pool<Any> {
        install_default_drivers();
        
        let unique_db_id = Uuid::new_v4().simple().to_string();
        let db_connection_string = format!("sqlite:file:memtest_opt_svc_{}?mode=memory&cache=shared", unique_db_id);
        
        let db_pool = AnyPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&db_connection_string)
            .await
            .expect("Failed to connect to test DB");

        // Create tables (same as in optimized repository tests)
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

        db_pool
    }

    #[tokio::test]
    async fn test_optimized_service_caching() {
        let service = OptimizedPaymentService::new();
        let db_pool = setup_test_db().await;
        let db_state = State::from(&db_pool);
        
        let payment = create_test_payment("CACHE_TEST");
        
        // Create payment
        let created = service.create_payment(&db_state, payment.clone()).await.unwrap();
        
        // First get - should cache the result
        let _first_get = service.get_payment_by_id(&db_state, &created.id).await.unwrap();
        
        // Second get - should use cache (faster)
        let _second_get = service.get_payment_by_id(&db_state, &created.id).await.unwrap();
        
        // Verify cache contains the payment
        let cached = service.get_cached_payment(&created.id).await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_optimized_service_validation() {
        let service = OptimizedPaymentService::new();
        let db_pool = setup_test_db().await;
        let db_state = State::from(&db_pool);
        
        // Test invalid payment (empty ID)
        let mut invalid_payment = create_test_payment("VALIDATION_TEST");
        invalid_payment.id = "".to_string();
        
        let result = service.create_payment(&db_state, invalid_payment).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            OptimizedPaymentError::ValidationError { errors } => {
                assert!(!errors.is_empty());
                assert!(errors.iter().any(|e| e.contains("Payment ID cannot be empty")));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_cached_parsing() {
        let service = OptimizedPaymentService::new();
        
        // Test cached method parsing
        assert_eq!(service.parse_payment_method("CASH").unwrap(), PaymentMethod::Cash);
        assert_eq!(service.parse_payment_method("cash").unwrap(), PaymentMethod::Cash);
        assert!(service.parse_payment_method("INVALID").is_err());
        
        // Test cached status parsing
        assert_eq!(service.parse_payment_status("LUNAS").unwrap(), PaymentStatus::Paid);
        assert_eq!(service.parse_payment_status("lunas").unwrap(), PaymentStatus::Paid);
        assert!(service.parse_payment_status("INVALID").is_err());
    }

    fn create_test_payment(suffix: &str) -> Payment {
        Payment {
            id: format!("PMT-OPT-SVC-{}", suffix),
            transaction_id: format!("TXN-OPT-SVC-{}", suffix),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: Some(Utc::now()),
        }
    }
}
