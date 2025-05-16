use std::collections::HashMap;
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

pub trait PaymentService: Send + Sync {
    fn create_payment(&self, transaction_id: String, amount: f64, method: PaymentMethod) -> Result<Payment, String>;
    fn update_payment_status(&self, payment_id: String, new_status: PaymentStatus, additional_amount: Option<f64>) -> Result<Payment, String>;
    fn delete_payment(&self, payment_id: String) -> Result<(), String>;
    fn get_payment(&self, payment_id: &str) -> Option<Payment>;
    fn get_payment_by_transaction(&self, transaction_id: &str) -> Option<Payment>;
    fn get_all_payments(&self, filters: Option<HashMap<String, String>>) -> Vec<Payment>;
    fn add_installment(&self, payment_id: &str, amount: f64) -> Result<Payment, String>;
}