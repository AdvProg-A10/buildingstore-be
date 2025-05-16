use crate::manajemen_pembayaran::model::payment::PaymentMethod;
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::patterns::strategy::{PaymentProcessor, CashPaymentProcessor, CreditCardPaymentProcessor, BankTransferPaymentProcessor, EWalletPaymentProcessor};
use crate::manajemen_pembayaran::patterns::state::{PaymentState, PaidState, InstallmentState};

pub struct PaymentProcessorFactory;
impl PaymentProcessorFactory {
    pub fn create(method: PaymentMethod) -> Box<dyn PaymentProcessor> {
        match method {
            PaymentMethod::Cash => Box::new(CashPaymentProcessor),
            PaymentMethod::CreditCard => Box::new(CreditCardPaymentProcessor),
            PaymentMethod::BankTransfer => Box::new(BankTransferPaymentProcessor),
            PaymentMethod::EWallet => Box::new(EWalletPaymentProcessor),
        }
    }
}

pub struct PaymentStateFactory;
impl PaymentStateFactory {
    pub fn create(status: &PaymentStatus) -> Box<dyn PaymentState> {
        match status {
            PaymentStatus::Paid => Box::new(PaidState),
            PaymentStatus::Installment => Box::new(InstallmentState),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_payment_processor_factory() {
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        for method in [
            PaymentMethod::Cash,
            PaymentMethod::CreditCard,
            PaymentMethod::BankTransfer,
            PaymentMethod::EWallet,
        ] {
            let processor = PaymentProcessorFactory::create(method.clone());
            
            assert_eq!(processor.get_method(), method);
            
            let result = processor.process(1000.0, &transaction_id);
            assert!(result.is_ok());
            
            let invalid_result = processor.process(-100.0, &transaction_id);
            assert!(invalid_result.is_err());
        }
    }
}