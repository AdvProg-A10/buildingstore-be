use std::sync::Arc;
use crate::manajemen_pembayaran::model::payment::Payment;

pub trait PaymentObserver: Send + Sync {
    fn payment_updated(&self, payment: &Payment);
}

pub struct TransactionObserver;
impl PaymentObserver for TransactionObserver {
    fn payment_updated(&self, payment: &Payment) {
        println!("Observer: Memperbarui status transaksi untuk ID {}", payment.transaction_id);
    }
}

pub struct InventoryObserver;
impl PaymentObserver for InventoryObserver {
    fn payment_updated(&self, payment: &Payment) {
        println!("Observer: Memperbarui stok untuk transaksi ID {}", payment.transaction_id);
    }
}

pub struct CustomerObserver;
impl PaymentObserver for CustomerObserver {
    fn payment_updated(&self, payment: &Payment) {
        println!("Observer: Memperbarui riwayat pelanggan untuk transaksi ID {}", payment.transaction_id);
    }
}

pub struct PaymentSubject {
    observers: Vec<Arc<dyn PaymentObserver>>,
}

impl PaymentSubject {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn attach(&mut self, observer: Arc<dyn PaymentObserver>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, payment: &Payment) {
        for observer in &self.observers {
            observer.payment_updated(payment);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use uuid::Uuid;
    use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
    use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

    struct MockObserver {
        notification_count: Mutex<usize>,
        last_payment_id: Mutex<Option<String>>,
    }

    impl MockObserver {
        fn new() -> Self {
            Self {
                notification_count: Mutex::new(0),
                last_payment_id: Mutex::new(None),
            }
        }
        
        fn get_notification_count(&self) -> usize {
            *self.notification_count.lock().unwrap()
        }
        
        fn get_last_payment_id(&self) -> Option<String> {
            self.last_payment_id.lock().unwrap().clone()
        }
    }

    impl PaymentObserver for MockObserver {
        fn payment_updated(&self, payment: &Payment) {
            let mut count = self.notification_count.lock().unwrap();
            *count += 1;
            
            let mut last_id = self.last_payment_id.lock().unwrap();
            *last_id = Some(payment.id.clone());
        }
    }

    #[test]
    fn test_observer_notification() {
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(MockObserver::new());
        
        subject.attach(observer.clone() as Arc<dyn PaymentObserver>);
        
        let payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        subject.notify(&payment);
        
        assert_eq!(observer.get_notification_count(), 1);
        assert_eq!(observer.get_last_payment_id(), Some(payment.id.clone()));
    }

    #[test]
    fn test_multiple_observers() {
        let mut subject = PaymentSubject::new();
        let observer1 = Arc::new(MockObserver::new());
        let observer2 = Arc::new(MockObserver::new());
        
        subject.attach(observer1.clone() as Arc<dyn PaymentObserver>);
        subject.attach(observer2.clone() as Arc<dyn PaymentObserver>);
        
        let payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        subject.notify(&payment);
        
        assert_eq!(observer1.get_notification_count(), 1);
        assert_eq!(observer2.get_notification_count(), 1);
    }

    #[test]
    fn test_concrete_observers() {
        let mut subject = PaymentSubject::new();
        
        let transaction_observer = Arc::new(TransactionObserver);
        let inventory_observer = Arc::new(InventoryObserver);
        let customer_observer = Arc::new(CustomerObserver);
        
        subject.attach(transaction_observer as Arc<dyn PaymentObserver>);
        subject.attach(inventory_observer as Arc<dyn PaymentObserver>);
        subject.attach(customer_observer as Arc<dyn PaymentObserver>);
        
        let payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        subject.notify(&payment);
        
        assert!(true);
    }
}
