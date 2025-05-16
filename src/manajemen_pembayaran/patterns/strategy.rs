use uuid::Uuid;
use crate::manajemen_pembayaran::model::payment::PaymentMethod;

pub trait PaymentProcessor: Send + Sync {
    fn process(&self, amount: f64, transaction_id: &str) -> Result<String, String>;
    fn get_method(&self) -> PaymentMethod;
}

pub struct CashPaymentProcessor;
impl PaymentProcessor for CashPaymentProcessor {
    fn process(&self, amount: f64, transaction_id: &str) -> Result<String, String> {
        println!("Memproses pembayaran tunai sebesar Rp{} untuk transaksi {}", amount, transaction_id);
        
        if amount <= 0.0 {
            return Err("Jumlah pembayaran tunai harus lebih dari 0".to_string());
        }
        Ok(format!("CASH-{}", Uuid::new_v4()))
    }

    fn get_method(&self) -> PaymentMethod {
        PaymentMethod::Cash
    }
}

pub struct CreditCardPaymentProcessor;
impl PaymentProcessor for CreditCardPaymentProcessor {
    fn process(&self, amount: f64, transaction_id: &str) -> Result<String, String> {
        println!("Memproses pembayaran kartu kredit sebesar Rp{} untuk transaksi {}", amount, transaction_id);
        
        if amount <= 0.0 {
            return Err("Jumlah pembayaran kartu kredit harus lebih dari 0".to_string());
        }
        Ok(format!("CC-{}", Uuid::new_v4()))
    }

    fn get_method(&self) -> PaymentMethod {
        PaymentMethod::CreditCard
    }
}

pub struct BankTransferPaymentProcessor;
impl PaymentProcessor for BankTransferPaymentProcessor {
    fn process(&self, amount: f64, transaction_id: &str) -> Result<String, String> {
        println!("Memproses pembayaran transfer bank sebesar Rp{} untuk transaksi {}", amount, transaction_id);
        
        if amount <= 0.0 {
            return Err("Jumlah pembayaran transfer bank harus lebih dari 0".to_string());
        }
        Ok(format!("BANK-{}", Uuid::new_v4()))
    }

    fn get_method(&self) -> PaymentMethod {
        PaymentMethod::BankTransfer
    }
}

pub struct EWalletPaymentProcessor;
impl PaymentProcessor for EWalletPaymentProcessor {
    fn process(&self, amount: f64, transaction_id: &str) -> Result<String, String> {
        println!("Memproses pembayaran e-wallet sebesar Rp{} untuk transaksi {}", amount, transaction_id);
        
        if amount <= 0.0 {
            return Err("Jumlah pembayaran e-wallet harus lebih dari 0".to_string());
        }
        Ok(format!("EWALLET-{}", Uuid::new_v4()))
    }

    fn get_method(&self) -> PaymentMethod {
        PaymentMethod::EWallet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_cash_payment_processor() {
        let processor = CashPaymentProcessor;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = processor.process(1000.0, &transaction_id);
        assert!(result.is_ok());
        let payment_id = result.unwrap();
        assert!(payment_id.starts_with("CASH-"));
        
        assert_eq!(processor.get_method(), PaymentMethod::Cash);
    }

    #[test]
    fn test_credit_card_payment_processor() {
        let processor = CreditCardPaymentProcessor;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = processor.process(1000.0, &transaction_id);
        assert!(result.is_ok());
        let payment_id = result.unwrap();
        assert!(payment_id.starts_with("CC-"));
        
        assert_eq!(processor.get_method(), PaymentMethod::CreditCard);
    }

    #[test]
    fn test_bank_transfer_payment_processor() {
        let processor = BankTransferPaymentProcessor;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = processor.process(1000.0, &transaction_id);
        assert!(result.is_ok());
        let payment_id = result.unwrap();
        assert!(payment_id.starts_with("BANK-"));
        
        assert_eq!(processor.get_method(), PaymentMethod::BankTransfer);
    }

    #[test]
    fn test_e_wallet_payment_processor() {
        let processor = EWalletPaymentProcessor;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = processor.process(1000.0, &transaction_id);
        assert!(result.is_ok());
        let payment_id = result.unwrap();
        assert!(payment_id.starts_with("EWALLET-"));
        
        assert_eq!(processor.get_method(), PaymentMethod::EWallet);
    }

    #[test]
    fn test_invalid_amount() {
        let processor = CashPaymentProcessor;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = processor.process(-100.0, &transaction_id);
        assert!(result.is_err());
    }
}