use uuid::Uuid;
use chrono::Utc;

use crate::manajemen_pembayaran::model::payment::{Payment, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

pub trait PaymentState: Send + Sync {
    fn process_payment(&self, payment: &mut Payment, amount: f64) -> Result<(), String>;
    fn can_delete(&self) -> bool;
    fn get_name(&self) -> String;
}

pub struct PaidState;
impl PaymentState for PaidState {
    fn process_payment(&self, _payment: &mut Payment, _amount: f64) -> Result<(), String> {
        Err("Pembayaran sudah lunas, tidak dapat menambahkan pembayaran lagi".to_string())
    }
    
    fn can_delete(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        "LUNAS".to_string()
    }
}

pub struct InstallmentState;
impl PaymentState for InstallmentState {
    fn process_payment(&self, payment: &mut Payment, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Jumlah cicilan harus lebih dari 0".to_string());
        }
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment.id.clone(),
            amount,
            payment_date: Utc::now(),
        };
        payment.installments.push(installment);

        let total_paid: f64 = payment.installments.iter().map(|i| i.amount).sum();
        if total_paid >= payment.amount {
            payment.status = PaymentStatus::Paid;
        }

        Ok(())
    }
    
    fn can_delete(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "CICILAN".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};

    #[test]
    fn test_paid_state_process_payment() {
        let state = PaidState;
        let mut payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let result = state.process_payment(&mut payment, 500.0);
        
        assert!(result.is_err());
        assert_eq!(payment.installments.len(), 0);
    }

    #[test]
    fn test_installment_state_process_payment() {
        let state = InstallmentState;
        let mut payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let result = state.process_payment(&mut payment, 400.0);
        
        assert!(result.is_ok()); 
        assert_eq!(payment.installments.len(), 1);
        assert_eq!(payment.installments[0].amount, 400.0);
        assert_eq!(payment.status, PaymentStatus::Installment);
        
        let result = state.process_payment(&mut payment, 600.0);
        
        assert!(result.is_ok());
        assert_eq!(payment.installments.len(), 2);
        assert_eq!(payment.status, PaymentStatus::Paid);
    }

    #[test]
    fn test_installment_state_invalid_amount() {
        let state = InstallmentState;
        let mut payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let result = state.process_payment(&mut payment, -100.0);
        
        assert!(result.is_err());
        assert_eq!(payment.installments.len(), 0);
    }

    #[test]
    fn test_state_can_delete() {
        let paid_state = PaidState;
        assert!(!paid_state.can_delete());
        
        let installment_state = InstallmentState;
        assert!(installment_state.can_delete());
    }
}