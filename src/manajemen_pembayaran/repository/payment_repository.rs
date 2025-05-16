use std::collections::HashMap;
use crate::manajemen_pembayaran::model::payment::Payment;

pub trait PaymentRepository: Send + Sync {
    fn save(&self, payment: Payment) -> Result<Payment, String>;
    fn find_by_id(&self, id: &str) -> Option<Payment>;
    fn find_by_transaction_id(&self, transaction_id: &str) -> Option<Payment>;
    fn find_all(&self, filters: Option<HashMap<String, String>>) -> Vec<Payment>;
    fn update(&self, payment: Payment) -> Result<Payment, String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}