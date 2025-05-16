use chrono::{DateTime, Utc};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    BankTransfer,
    EWallet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    }
}