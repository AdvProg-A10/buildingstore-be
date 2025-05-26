use chrono::{DateTime, Utc};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    BankTransfer,
    EWallet,
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "CASH"),
            PaymentMethod::CreditCard => write!(f, "CREDIT_CARD"),
            PaymentMethod::BankTransfer => write!(f, "BANK_TRANSFER"),
            PaymentMethod::EWallet => write!(f, "E_WALLET"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Payment {
    pub id: String,
    pub transaction_id: String,
    pub amount: f64,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub payment_date: DateTime<Utc>,
    pub installments: Vec<Installment>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Installment {
    pub id: String,
    pub payment_id: String,
    pub amount: f64,
    pub payment_date: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_payment_method_display() {
        assert_eq!(PaymentMethod::Cash.to_string(), "CASH");
        assert_eq!(PaymentMethod::CreditCard.to_string(), "CREDIT_CARD");
        assert_eq!(PaymentMethod::BankTransfer.to_string(), "BANK_TRANSFER");
        assert_eq!(PaymentMethod::EWallet.to_string(), "E_WALLET");
    }

    #[test]
    fn test_payment_method_equality() {
        let method1 = PaymentMethod::Cash;
        let method2 = PaymentMethod::Cash;
        let method3 = PaymentMethod::CreditCard;
        
        assert_eq!(method1, method2);
        assert_ne!(method1, method3);
    }

    #[test]
    fn test_payment_status_from_string() {
        let paid_status = PaymentStatus::from_string("LUNAS");
        assert!(paid_status.is_some());
        assert_eq!(paid_status.unwrap(), PaymentStatus::Paid);

        let installment_status = PaymentStatus::from_string("CICILAN");
        assert!(installment_status.is_some());
        assert_eq!(installment_status.unwrap(), PaymentStatus::Installment);

        let lowercase_status = PaymentStatus::from_string("lunas");
        assert!(lowercase_status.is_some());
        assert_eq!(lowercase_status.unwrap(), PaymentStatus::Paid);

        let invalid_status = PaymentStatus::from_string("INVALID");
        assert!(invalid_status.is_none());
    }

    #[test]
    fn test_payment_status_to_string() {
        assert_eq!(PaymentStatus::Paid.to_string(), "LUNAS");
        assert_eq!(PaymentStatus::Installment.to_string(), "CICILAN");
    }

    #[test]
    fn test_create_payment() {
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        assert_eq!(payment.id, payment_id);
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 1000.0);
        assert_eq!(payment.method, PaymentMethod::Cash);
        assert_eq!(payment.status, PaymentStatus::Paid);
        assert!(payment.installments.is_empty());
    }

    #[test]
    fn test_create_payment_with_due_date() {
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let due_date = Utc::now() + chrono::Duration::days(30);
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: transaction_id.clone(),
            amount: 2000.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: Some(due_date),
        };
        
        assert_eq!(payment.id, payment_id);
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 2000.0);
        assert_eq!(payment.method, PaymentMethod::CreditCard);
        assert_eq!(payment.status, PaymentStatus::Installment);
        assert!(payment.due_date.is_some());
    }

    #[test]
    fn test_add_installment() {
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let mut payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.clone(),
            amount: 500.0,
            payment_date: Utc::now(),
        };
        
        payment.installments.push(installment);
        
        assert_eq!(payment.installments.len(), 1);
        assert_eq!(payment.installments[0].amount, 500.0);
        assert_eq!(payment.installments[0].payment_id, payment_id);
    }

    #[test]
    fn test_add_multiple_installments() {
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let mut payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let installment1 = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.clone(),
            amount: 300.0,
            payment_date: Utc::now(),
        };
        
        let installment2 = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.clone(),
            amount: 400.0,
            payment_date: Utc::now() + chrono::Duration::days(1),
        };
        
        payment.installments.push(installment1);
        payment.installments.push(installment2);
        
        assert_eq!(payment.installments.len(), 2);
        
        let total_installments: f64 = payment.installments.iter().map(|i| i.amount).sum();
        assert_eq!(total_installments, 700.0);
    }

    #[test]
    fn test_installment_creation() {
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        let installment_id = format!("INST-{}", Uuid::new_v4());
        let payment_date = Utc::now();
        
        let installment = Installment {
            id: installment_id.clone(),
            payment_id: payment_id.clone(),
            amount: 250.0,
            payment_date,
        };
        
        assert_eq!(installment.id, installment_id);
        assert_eq!(installment.payment_id, payment_id);
        assert_eq!(installment.amount, 250.0);
        assert_eq!(installment.payment_date, payment_date);
    }

    #[test]
    fn test_payment_serialization() {
        let payment = Payment {
            id: "PMT-123".to_string(),
            transaction_id: "TRX-456".to_string(),
            amount: 1000.0,
            method: PaymentMethod::EWallet,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let serialized = serde_json::to_string(&payment);
        assert!(serialized.is_ok());
        
        let json_str = serialized.unwrap();
        assert!(json_str.contains("PMT-123"));
        assert!(json_str.contains("TRX-456"));
        assert!(json_str.contains("1000"));
    }

    #[test]
    fn test_installment_serialization() {
        let installment = Installment {
            id: "INST-789".to_string(),
            payment_id: "PMT-123".to_string(),
            amount: 500.0,
            payment_date: Utc::now(),
        };
        
        let serialized = serde_json::to_string(&installment);
        assert!(serialized.is_ok());
        
        let json_str = serialized.unwrap();
        assert!(json_str.contains("INST-789"));
        assert!(json_str.contains("PMT-123"));
        assert!(json_str.contains("500"));
    }
}