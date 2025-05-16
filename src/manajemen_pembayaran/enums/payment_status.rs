use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Paid,    // LUNAS
    Installment,  // CICILAN
}

impl PaymentStatus {
    pub fn from_string(status: &str) -> Option<Self> {
        match status.to_uppercase().as_str() {
            "LUNAS" => Some(PaymentStatus::Paid),
            "CICILAN" => Some(PaymentStatus::Installment),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            PaymentStatus::Paid => "LUNAS".to_string(),
            PaymentStatus::Installment => "CICILAN".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status_from_string() {
        // Valid status
        let paid_status = PaymentStatus::from_string("LUNAS");
        assert!(paid_status.is_some());
        assert_eq!(paid_status.unwrap(), PaymentStatus::Paid);

        let installment_status = PaymentStatus::from_string("CICILAN");
        assert!(installment_status.is_some());
        assert_eq!(installment_status.unwrap(), PaymentStatus::Installment);

        // Case insensitive
        let lowercase_status = PaymentStatus::from_string("lunas");
        assert!(lowercase_status.is_some());
        assert_eq!(lowercase_status.unwrap(), PaymentStatus::Paid);

        // Invalid status
        let invalid_status = PaymentStatus::from_string("INVALID");
        assert!(invalid_status.is_none());
    }

    #[test]
    fn test_payment_status_to_string() {
        assert_eq!(PaymentStatus::Paid.to_string(), "LUNAS");
        assert_eq!(PaymentStatus::Installment.to_string(), "CICILAN");
    }
}