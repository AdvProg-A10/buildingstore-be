use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::manajemen_pembayaran::model::payment::Payment;
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;

pub struct PaymentRepositoryImpl {
    payments: Arc<Mutex<HashMap<String, Payment>>>,
}

impl PaymentRepositoryImpl {
    pub fn new() -> Self {
        Self {
            payments: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl PaymentRepository for PaymentRepositoryImpl {
    fn save(&self, payment: Payment) -> Result<Payment, String> {
        let mut payments = self.payments.lock().unwrap();
        let payment_clone = payment.clone();
        payments.insert(payment.id.clone(), payment);
        Ok(payment_clone)
    }

    fn find_by_id(&self, id: &str) -> Option<Payment> {
        let payments = self.payments.lock().unwrap();
        payments.get(id).cloned()
    }

    fn find_by_transaction_id(&self, transaction_id: &str) -> Option<Payment> {
        let payments = self.payments.lock().unwrap();
        payments
            .values()
            .find(|p| p.transaction_id == transaction_id)
            .cloned()
    }

    fn find_all(&self, filters: Option<HashMap<String, String>>) -> Vec<Payment> {
        let payments = self.payments.lock().unwrap();
        let mut result: Vec<Payment> = payments.values().cloned().collect();

        if let Some(filter_map) = filters {
            // Filter by status
            if let Some(status_str) = filter_map.get("status") {
                if let Some(status) = PaymentStatus::from_string(status_str) {
                    result = result
                        .into_iter()
                        .filter(|p| p.status == status)
                        .collect();
                }
            }
            
            // Filter by method
            if let Some(method) = filter_map.get("method") {
                result = result
                    .into_iter()
                    .filter(|p| format!("{:?}", p.method).to_uppercase() == method.to_uppercase())
                    .collect();
            }
        }

        result
    }

    fn update(&self, payment: Payment) -> Result<Payment, String> {
        let mut payments = self.payments.lock().unwrap();
        if !payments.contains_key(&payment.id) {
            return Err(format!("Pembayaran dengan ID {} tidak ditemukan", payment.id));
        }
        let payment_clone = payment.clone();
        payments.insert(payment.id.clone(), payment);
        Ok(payment_clone)
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let mut payments = self.payments.lock().unwrap();
        if payments.remove(id).is_none() {
            return Err(format!("Pembayaran dengan ID {} tidak ditemukan", id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};

    #[test]
    fn test_save_payment() {
        let repository = PaymentRepositoryImpl::new();
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
        
        let result = repository.save(payment.clone());
        
        assert!(result.is_ok());
        let saved_payment = result.unwrap();
        assert_eq!(saved_payment.id, payment_id);
        assert_eq!(saved_payment.transaction_id, transaction_id);
        assert_eq!(saved_payment.amount, 1000.0);
    }

    #[test]
    fn test_find_by_id() {
        let repository = PaymentRepositoryImpl::new();
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(payment.clone()).unwrap();
        
        let result = repository.find_by_id(&payment_id);
        
        assert!(result.is_some());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.id, payment_id);
    }

    #[test]
    fn test_find_by_transaction_id() {
        let repository = PaymentRepositoryImpl::new();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(payment.clone()).unwrap();
        
        let result = repository.find_by_transaction_id(&transaction_id);
        
        assert!(result.is_some());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.transaction_id, transaction_id);
    }

    #[test]
    fn test_update_payment() {
        let repository = PaymentRepositoryImpl::new();
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(payment.clone()).unwrap();
        
        let mut updated_payment = payment.clone();
        updated_payment.status = PaymentStatus::Paid;
        let result = repository.update(updated_payment);
        
        assert!(result.is_ok());
        let found = repository.find_by_id(&payment_id).unwrap();
        assert_eq!(found.status, PaymentStatus::Paid);
    }

    #[test]
    fn test_delete_payment() {
        let repository = PaymentRepositoryImpl::new();
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(payment.clone()).unwrap();
        
        let result = repository.delete(&payment_id);
        
        assert!(result.is_ok());
        assert!(repository.find_by_id(&payment_id).is_none());
    }

    #[test]
    fn test_find_all_with_filters() {
        let repository = PaymentRepositoryImpl::new();
        
        let cash_payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let credit_payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 2000.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(cash_payment.clone()).unwrap();
        repository.save(credit_payment.clone()).unwrap();
        
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        let status_results = repository.find_all(Some(filters));
        
        assert_eq!(status_results.len(), 1);
        assert_eq!(status_results[0].status, PaymentStatus::Paid);
        
        let all_results = repository.find_all(None);
        
        assert_eq!(all_results.len(), 2);
    }
}