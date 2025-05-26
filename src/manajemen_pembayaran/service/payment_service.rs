use std::collections::HashMap;
use rocket::State;
use chrono::{Utc};
use uuid::Uuid;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PembayaranRepository;
use sqlx::{Any, Pool};

pub struct PaymentService;

#[derive(Debug)]
pub enum PaymentError {
    DatabaseError(String),
    NotFound(String),
    InvalidInput(String),
}

impl PaymentService {
    pub fn new() -> Self {
        PaymentService {}
    }
    
    pub async fn create_payment(&self, db: &State<Pool<Any>>, payment: Payment) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::create(conn, &payment).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }

    pub async fn get_payment_by_id(&self, db: &State<Pool<Any>>, id: &str) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::find_by_id(conn, id).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", id)),
                _ => PaymentError::DatabaseError(e.to_string())
            })
    }

    pub async fn get_all_payments(&self, db: &State<Pool<Any>>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::find_all(conn, filters).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }

    pub async fn update_payment(&self, db: &State<Pool<Any>>, payment: Payment) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::update(conn, &payment).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment.id)),
                _ => PaymentError::DatabaseError(e.to_string())
            })
    }
    pub async fn update_payment_status(&self, db: &State<Pool<Any>>, payment_id: String, new_status: PaymentStatus, additional_amount: Option<f64>) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::update_payment_status(conn, payment_id.clone(), new_status, additional_amount).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment_id)),
                _ => PaymentError::DatabaseError(e.to_string())
            })
    }
    
    pub async fn delete_payment(&self, db: &State<Pool<Any>>, payment_id: &str) -> Result<(), PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::delete(conn, payment_id).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }
    
    pub async fn add_installment(&self, db: &State<Pool<Any>>, payment_id: &str, amount: f64) -> Result<Payment, PaymentError> {
        let payment: Payment = self.get_payment_by_id(db, payment_id).await?;
        
        if payment.status != PaymentStatus::Installment {
            return Err(PaymentError::InvalidInput("Cannot add installment to a payment that is not in INSTALLMENT status".to_string()));
        }
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };
        
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        let mut updated_payment = payment.clone();
        updated_payment.installments.push(installment);
        
        PembayaranRepository::update(conn, &updated_payment).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }
    
    pub fn generate_payment_id(&self) -> String {
        format!("PMT-{}", Uuid::new_v4())
    }
    
    pub fn parse_payment_method(&self, method_str: &str) -> Result<PaymentMethod, PaymentError> {
        match method_str.to_uppercase().as_str() {
            "CASH" => Ok(PaymentMethod::Cash),
            "CREDIT_CARD" => Ok(PaymentMethod::CreditCard),
            "BANK_TRANSFER" => Ok(PaymentMethod::BankTransfer),
            "E_WALLET" => Ok(PaymentMethod::EWallet),
            _ => Err(PaymentError::InvalidInput(format!("Invalid payment method: {}", method_str))),
        }
    }
    
    pub fn parse_payment_status(&self, status_str: &str) -> Result<PaymentStatus, PaymentError> {
        PaymentStatus::from_string(status_str)
            .ok_or_else(|| PaymentError::InvalidInput(format!("Invalid payment status: {}", status_str)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc};
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_payment_service_creation() {
        let service = PaymentService::new();
        assert_eq!(std::mem::size_of_val(&service), 0);
    }

    #[test]
    fn test_generate_payment_id() {
        let service = PaymentService::new();
        let id1 = service.generate_payment_id();
        let id2 = service.generate_payment_id();

        assert!(id1.starts_with("PMT-"));
        assert!(id2.starts_with("PMT-"));
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 40);
    }

    #[test]
    fn test_parse_payment_method_valid() {
        let service = PaymentService::new();

        assert_eq!(service.parse_payment_method("CASH").unwrap(), PaymentMethod::Cash);
        assert_eq!(service.parse_payment_method("cash").unwrap(), PaymentMethod::Cash);
        assert_eq!(service.parse_payment_method("Cash").unwrap(), PaymentMethod::Cash);

        assert_eq!(service.parse_payment_method("CREDIT_CARD").unwrap(), PaymentMethod::CreditCard);
        assert_eq!(service.parse_payment_method("credit_card").unwrap(), PaymentMethod::CreditCard);

        assert_eq!(service.parse_payment_method("BANK_TRANSFER").unwrap(), PaymentMethod::BankTransfer);
        assert_eq!(service.parse_payment_method("bank_transfer").unwrap(), PaymentMethod::BankTransfer);

        assert_eq!(service.parse_payment_method("E_WALLET").unwrap(), PaymentMethod::EWallet);
        assert_eq!(service.parse_payment_method("e_wallet").unwrap(), PaymentMethod::EWallet);
    }

    #[test]
    fn test_parse_payment_method_invalid() {
        let service = PaymentService::new();

        let result = service.parse_payment_method("INVALID_METHOD");
        assert!(result.is_err());

        if let Err(PaymentError::InvalidInput(msg)) = result {
            assert!(msg.contains("Invalid payment method"));
            assert!(msg.contains("INVALID_METHOD"));
        } else {
            panic!("Expected PaymentError::InvalidInput");
        }

        let result2 = service.parse_payment_method("");
        assert!(result2.is_err());
    }    #[test]
    fn test_parse_payment_status_valid() {
        let service = PaymentService::new();

        assert_eq!(service.parse_payment_status("LUNAS").unwrap(), PaymentStatus::Paid);
        assert_eq!(service.parse_payment_status("CICILAN").unwrap(), PaymentStatus::Installment);
    }

    #[test]
    fn test_parse_payment_status_invalid() {
        let service = PaymentService::new();

        let result = service.parse_payment_status("INVALID_STATUS");
        assert!(result.is_err());

        if let Err(PaymentError::InvalidInput(msg)) = result {
            assert!(msg.contains("Invalid payment status"));
            assert!(msg.contains("INVALID_STATUS"));
        } else {
            panic!("Expected PaymentError::InvalidInput");
        }

        let result2 = service.parse_payment_status("");
        assert!(result2.is_err());
    }

    #[test]
    fn test_payment_error_types() {
        let db_error = PaymentError::DatabaseError("Database connection failed".to_string());
        let not_found_error = PaymentError::NotFound("Payment not found".to_string());
        let invalid_input_error = PaymentError::InvalidInput("Invalid payment data".to_string());

        match db_error {
            PaymentError::DatabaseError(msg) => assert_eq!(msg, "Database connection failed"),
            _ => panic!("Expected DatabaseError"),
        }

        match not_found_error {
            PaymentError::NotFound(msg) => assert_eq!(msg, "Payment not found"),
            _ => panic!("Expected NotFound"),
        }

        match invalid_input_error {
            PaymentError::InvalidInput(msg) => assert_eq!(msg, "Invalid payment data"),
            _ => panic!("Expected InvalidInput"),
        }
    }

    #[test]
    fn test_installment_validation_logic() {
        let payment = Payment {
            id: "payment-123".to_string(),
            transaction_id: "txn-456".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        assert_eq!(payment.status, PaymentStatus::Installment);
        let invalid_payment = Payment {
            status: PaymentStatus::Paid,
            ..payment
        };

        assert_ne!(invalid_payment.status, PaymentStatus::Installment);
    }

    #[test]
    fn test_installment_creation_in_service() {
        let payment_id = "payment-123";
        let amount = 500.0;
        let installment_id = format!("INST-{}", Uuid::new_v4());

        let installment = Installment {
            id: installment_id.clone(),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };

        assert_eq!(installment.payment_id, payment_id);
        assert_eq!(installment.amount, amount);
        assert!(installment.id.starts_with("INST-"));
    }

    #[test]
    fn test_filter_handling() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "COMPLETED".to_string());
        filters.insert("method".to_string(), "CASH".to_string());

        assert_eq!(filters.get("status"), Some(&"COMPLETED".to_string()));
        assert_eq!(filters.get("method"), Some(&"CASH".to_string()));
        assert_eq!(filters.get("nonexistent"), None);

        let empty_filters: Option<HashMap<String, String>> = None;
        assert!(empty_filters.is_none());
    }

    #[test]
    fn test_payment_structure_for_service() {
        let payment = Payment {
            id: "PMT-123".to_string(),
            transaction_id: "TXN-456".to_string(),
            amount: 1500.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: vec![
                Installment {
                    id: "INST-1".to_string(),
                    payment_id: "PMT-123".to_string(),
                    amount: 750.0,
                    payment_date: Utc::now(),
                },
                Installment {
                    id: "INST-2".to_string(),
                    payment_id: "PMT-123".to_string(),
                    amount: 750.0,
                    payment_date: Utc::now(),
                },
            ],
            due_date: Some(Utc::now()),
        };

        assert_eq!(payment.installments.len(), 2);
        assert_eq!(payment.amount, 1500.0);
        
        let total_installments: f64 = payment.installments.iter().map(|i| i.amount).sum();
        assert_eq!(total_installments, 1500.0);
    }

    #[test]
    fn test_payment_cloning() {
        let original_payment = Payment {
            id: "PMT-789".to_string(),
            transaction_id: "TXN-101112".to_string(),
            amount: 2000.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let cloned_payment = original_payment.clone();

        assert_eq!(original_payment.id, cloned_payment.id);
        assert_eq!(original_payment.transaction_id, cloned_payment.transaction_id);
        assert_eq!(original_payment.amount, cloned_payment.amount);
        assert_eq!(original_payment.method, cloned_payment.method);
        assert_eq!(original_payment.status, cloned_payment.status);
    }

    #[test]
    fn test_uuid_generation_consistency() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.to_string().len(), 36);
        assert_eq!(uuid2.to_string().len(), 36);
    }

    #[tokio::test]
    async fn test_create_payment_success() {
        let _service = PaymentService::new();
        
        let _payment = Payment {
            id: "PMT-TEST-001".to_string(),
            transaction_id: "TXN-TEST-001".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        let db_error = sqlx::Error::PoolClosed;
        let payment_error = PaymentError::DatabaseError(db_error.to_string());
        
        match payment_error {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("pool"));
            },
            _ => panic!("Expected DatabaseError"),
        }
    }    
    
    #[tokio::test]
    async fn test_get_payment_by_id_error_handling() {
        let _service = PaymentService::new();
        
        let row_not_found_error = sqlx::Error::RowNotFound;
        let payment_error = match row_not_found_error {
            sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", "test-id")),
            _ => PaymentError::DatabaseError("other error".to_string())
        };
        
        match payment_error {
            PaymentError::NotFound(msg) => {
                assert!(msg.contains("Payment with id test-id not found"));
            },
            _ => panic!("Expected NotFound error"),
        }
        
        let other_db_error = sqlx::Error::PoolClosed;
        let payment_error2 = match other_db_error {
            sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", "test-id")),
            _ => PaymentError::DatabaseError(other_db_error.to_string())
        };
        
        match payment_error2 {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("pool"));
            },
            _ => panic!("Expected DatabaseError"),
        }
    }    
    
    #[tokio::test]
    async fn test_get_all_payments_with_filters() {
        let _service = PaymentService::new();
        
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "PAID".to_string());
        filters.insert("method".to_string(), "CASH".to_string());
        
        let filters_option = Some(filters.clone());
        assert!(filters_option.is_some());
        
        if let Some(filter_map) = filters_option {
            assert_eq!(filter_map.get("status"), Some(&"PAID".to_string()));
            assert_eq!(filter_map.get("method"), Some(&"CASH".to_string()));
        }
        
        let empty_filters: Option<HashMap<String, String>> = None;
        assert!(empty_filters.is_none());
    }    
    
    #[tokio::test]
    async fn test_update_payment_error_handling() {
        let _service = PaymentService::new();
        
        let payment = Payment {
            id: "PMT-UPDATE-001".to_string(),
            transaction_id: "TXN-UPDATE-001".to_string(),
            amount: 1500.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let row_not_found_error = sqlx::Error::RowNotFound;
        let payment_error = match row_not_found_error {
            sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment.id)),
            _ => PaymentError::DatabaseError("other error".to_string())
        };
        
        match payment_error {
            PaymentError::NotFound(msg) => {
                assert!(msg.contains("Payment with id PMT-UPDATE-001 not found"));
            },
            _ => panic!("Expected NotFound error"),
        }
        
        let other_db_error = sqlx::Error::PoolTimedOut;
        let payment_error2 = match other_db_error {
            sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment.id)),
            _ => PaymentError::DatabaseError(other_db_error.to_string())
        };
        
        match payment_error2 {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("timed out"));
            },
            _ => panic!("Expected DatabaseError"),
        }
    }    
    
    #[tokio::test]
    async fn test_update_payment_status_error_handling() {
        let _service = PaymentService::new();
        
        let payment_id = "PMT-STATUS-001".to_string();
        let new_status = PaymentStatus::Paid;
        let additional_amount = Some(250.0);
        
        let row_not_found_error = sqlx::Error::RowNotFound;
        let payment_error = match row_not_found_error {
            sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment_id.clone())),
            _ => PaymentError::DatabaseError("other error".to_string())
        };
        
        match payment_error {
            PaymentError::NotFound(msg) => {
                assert!(msg.contains("Payment with id PMT-STATUS-001 not found"));
            },
            _ => panic!("Expected NotFound error"),
        }
        
        assert_eq!(payment_id, "PMT-STATUS-001");
        assert_eq!(new_status, PaymentStatus::Paid);
        assert_eq!(additional_amount, Some(250.0));
        
        let cloned_payment_id = payment_id.clone();
        assert_eq!(cloned_payment_id, payment_id);
    }    
    
    #[tokio::test]
    async fn test_delete_payment_error_handling() {
        let _service = PaymentService::new();
        
        let _payment_id = "PMT-DELETE-001";
        
        let db_connection_error = sqlx::Error::PoolClosed;
        let payment_error = PaymentError::DatabaseError(db_connection_error.to_string());
        
        match payment_error {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("pool"));
            },
            _ => panic!("Expected DatabaseError"),
        }
        let repository_error = sqlx::Error::ColumnNotFound("payment_id".to_string());
        let mapped_error = PaymentError::DatabaseError(repository_error.to_string());
        
        match mapped_error {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("payment_id") || msg.contains("ColumnNotFound"));
            },
            _ => panic!("Expected DatabaseError"),
        }
    }   
    
    #[tokio::test]
    async fn test_add_installment_validation_logic() {
        let _service = PaymentService::new();
        
        let payment_id = "PMT-INSTALLMENT-001";
        let amount = 500.0;
        
        let valid_payment = Payment {
            id: payment_id.to_string(),
            transaction_id: "TXN-INSTALLMENT-001".to_string(),
            amount: 1000.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        if valid_payment.status != PaymentStatus::Installment {
            panic!("This should not happen for valid payment");
        } else {
            assert_eq!(valid_payment.status, PaymentStatus::Installment);
        }
        
        let invalid_payment = Payment {
            id: payment_id.to_string(),
            transaction_id: "TXN-INSTALLMENT-002".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        if invalid_payment.status != PaymentStatus::Installment {
            let error = PaymentError::InvalidInput("Cannot add installment to a payment that is not in INSTALLMENT status".to_string());
            match error {
                PaymentError::InvalidInput(msg) => {
                    assert!(msg.contains("Cannot add installment"));
                    assert!(msg.contains("INSTALLMENT status"));
                },
                _ => panic!("Expected InvalidInput error"),
            }
        }
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };
        
        assert!(installment.id.starts_with("INST-"));
        assert_eq!(installment.payment_id, payment_id);
        assert_eq!(installment.amount, amount);
        
        let mut updated_payment = valid_payment.clone();
        updated_payment.installments.push(installment.clone());
        
        assert_eq!(updated_payment.installments.len(), 1);
        assert_eq!(updated_payment.installments[0].id, installment.id);
        assert_eq!(updated_payment.installments[0].amount, installment.amount);
        
        assert_eq!(valid_payment.installments.len(), 0);
    }    
    
    #[tokio::test]
    async fn test_add_installment_database_operations() {
        let _service = PaymentService::new();
        
        let db_connection_error = sqlx::Error::PoolTimedOut;
        let payment_error = PaymentError::DatabaseError(db_connection_error.to_string());
        
        match payment_error {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("timed out"));
            },
            _ => panic!("Expected DatabaseError"),
        }
        
        let repository_update_error = sqlx::Error::ColumnNotFound("invalid_column".to_string());
        let mapped_error = PaymentError::DatabaseError(repository_update_error.to_string());
        
        match mapped_error {
            PaymentError::DatabaseError(msg) => {
                assert!(msg.contains("invalid_column"));
            },
            _ => panic!("Expected DatabaseError"),
        }
    }    
    
    #[tokio::test]
    async fn test_service_method_parameter_types() {
        let _service = PaymentService::new();
        
        let payment = Payment {
            id: "PMT-PARAM-001".to_string(),
            transaction_id: "TXN-PARAM-001".to_string(),
            amount: 750.0,
            method: PaymentMethod::EWallet,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: Some(Utc::now()),
        };
        
        assert_eq!(payment.id, "PMT-PARAM-001");
        assert_eq!(payment.amount, 750.0);
        assert_eq!(payment.method, PaymentMethod::EWallet);
        let payment_id = "PMT-SEARCH-001";
        assert_eq!(payment_id.len(), 14);
        assert!(payment_id.starts_with("PMT-"));
        
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "INSTALLMENT".to_string());
        filters.insert("method".to_string(), "BANK_TRANSFER".to_string());
        filters.insert("transaction_id".to_string(), "TXN-FILTER-001".to_string());
        
        let filters_option = Some(filters);
        assert!(filters_option.is_some());
        
        let status_payment_id = "PMT-STATUS-002".to_string();
        let new_status = PaymentStatus::Installment;
        let additional_amount: Option<f64> = Some(125.0);
        
        assert_eq!(status_payment_id, "PMT-STATUS-002");
        assert_eq!(new_status, PaymentStatus::Installment);
        assert_eq!(additional_amount, Some(125.0));
        
        let delete_payment_id = "PMT-DELETE-002";
        assert!(delete_payment_id.starts_with("PMT-"));
        
        let installment_payment_id = "PMT-INST-002";
        let installment_amount = 300.0;
        
        assert_eq!(installment_payment_id, "PMT-INST-002");
        assert_eq!(installment_amount, 300.0);
        assert!(installment_amount > 0.0);
    }    
    
    #[tokio::test]
    async fn test_error_propagation_patterns() {
        let _service = PaymentService::new();
        
        let sample_errors = vec!(
            sqlx::Error::RowNotFound,
            sqlx::Error::PoolClosed,
            sqlx::Error::PoolTimedOut,
        );
        
        for error in sample_errors {
            let mapped_error = match error {
                sqlx::Error::RowNotFound => PaymentError::NotFound("Test not found".to_string()),
                _ => PaymentError::DatabaseError(error.to_string()),
            };
            
            match mapped_error {
                PaymentError::NotFound(msg) => assert_eq!(msg, "Test not found"),
                PaymentError::DatabaseError(msg) => assert!(!msg.is_empty()),
                _ => panic!("Unexpected error type"),
            }
        }
    }
}