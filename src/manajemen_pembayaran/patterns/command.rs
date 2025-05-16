use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;
use crate::manajemen_pembayaran::patterns::observer::PaymentSubject;
use crate::manajemen_pembayaran::patterns::factory::{PaymentProcessorFactory, PaymentStateFactory};

pub trait PaymentCommand {
    fn execute(&self) -> Result<Payment, String>;
}

pub struct CreatePaymentCommand {
    transaction_id: String,
    amount: f64,
    method: PaymentMethod,
    repository: Arc<dyn PaymentRepository>,
    subject: Arc<PaymentSubject>,
}

impl CreatePaymentCommand {
    pub fn new(
        transaction_id: String,
        amount: f64,
        method: PaymentMethod,
        repository: Arc<dyn PaymentRepository>,
        subject: Arc<PaymentSubject>,
    ) -> Self {
        Self {
            transaction_id,
            amount,
            method,
            repository,
            subject,
        }
    }
}

impl PaymentCommand for CreatePaymentCommand {
    fn execute(&self) -> Result<Payment, String> {
        if self.repository.find_by_transaction_id(&self.transaction_id).is_some() {
            return Err(format!("Transaksi {} sudah memiliki pembayaran", self.transaction_id));
        }

        let processor = PaymentProcessorFactory::create(self.method.clone());
        let payment_id = processor.process(self.amount, &self.transaction_id)?;

        let payment = Payment {
            id: payment_id,
            transaction_id: self.transaction_id.clone(),
            amount: self.amount,
            method: processor.get_method(),
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let saved_payment = self.repository.save(payment)?;
        
        self.subject.notify(&saved_payment);
        
        Ok(saved_payment)
    }
}

pub struct UpdatePaymentStatusCommand {
    payment_id: String,
    new_status: PaymentStatus,
    additional_amount: Option<f64>,
    repository: Arc<dyn PaymentRepository>,
    subject: Arc<PaymentSubject>,
}

impl UpdatePaymentStatusCommand {
    pub fn new(
        payment_id: String,
        new_status: PaymentStatus,
        additional_amount: Option<f64>,
        repository: Arc<dyn PaymentRepository>,
        subject: Arc<PaymentSubject>,
    ) -> Self {
        Self {
            payment_id,
            new_status,
            additional_amount,
            repository,
            subject,
        }
    }
}

impl PaymentCommand for UpdatePaymentStatusCommand {
    fn execute(&self) -> Result<Payment, String> {
        let mut payment = self.repository
            .find_by_id(&self.payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", self.payment_id))?;

        payment.status = self.new_status.clone();

        if let Some(amount) = self.additional_amount {
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
        }

        let updated_payment = self.repository.update(payment)?;
        
        self.subject.notify(&updated_payment);
        
        Ok(updated_payment)
    }
}

pub struct AddInstallmentCommand {
    payment_id: String,
    amount: f64,
    repository: Arc<dyn PaymentRepository>,
    subject: Arc<PaymentSubject>,
}

impl AddInstallmentCommand {
    pub fn new(
        payment_id: String,
        amount: f64,
        repository: Arc<dyn PaymentRepository>,
        subject: Arc<PaymentSubject>,
    ) -> Self {
        Self {
            payment_id,
            amount,
            repository,
            subject,
        }
    }
}

impl PaymentCommand for AddInstallmentCommand {
    fn execute(&self) -> Result<Payment, String> {
        let mut payment = self.repository
            .find_by_id(&self.payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", self.payment_id))?;

        let state = PaymentStateFactory::create(&payment.status);
        state.process_payment(&mut payment, self.amount)?;

        let updated_payment = self.repository.update(payment)?;
        self.subject.notify(&updated_payment);
        
        Ok(updated_payment)
    }
}

pub struct DeletePaymentCommand {
    payment_id: String,
    repository: Arc<dyn PaymentRepository>,
}

impl DeletePaymentCommand {
    pub fn new(payment_id: String, repository: Arc<dyn PaymentRepository>) -> Self {
        Self {
            payment_id,
            repository,
        }
    }
}

impl DeletePaymentCommand {
    pub fn execute(&self) -> Result<(), String> {
        let payment = self.repository.find_by_id(&self.payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", self.payment_id))?;
            
        let state = PaymentStateFactory::create(&payment.status);
        if !state.can_delete() {
            return Err(format!("Tidak dapat menghapus pembayaran dengan status {}", state.get_name()));
        }
        
        self.repository.delete(&self.payment_id)
    }
}