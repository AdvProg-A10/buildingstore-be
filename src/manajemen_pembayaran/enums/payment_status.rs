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
    fn test_payment_status_case_variations() {
        let test_cases = vec![
            ("LUNAS", Some(PaymentStatus::Paid)),
            ("lunas", Some(PaymentStatus::Paid)),
            ("Lunas", Some(PaymentStatus::Paid)),
            ("LuNaS", Some(PaymentStatus::Paid)),
            ("CICILAN", Some(PaymentStatus::Installment)),
            ("cicilan", Some(PaymentStatus::Installment)),
            ("Cicilan", Some(PaymentStatus::Installment)),
            ("CiCiLaN", Some(PaymentStatus::Installment)),
        ];

        for (input, expected) in test_cases {
            let result = PaymentStatus::from_string(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_payment_status_invalid_inputs() {
        let invalid_inputs = vec![
            "",
            " ",
            "PENDING",
            "COMPLETED",
            "FAILED",
            "CANCELLED",
            "PARTIAL",
            "123",
            "null",
            "undefined",
        ];

        for input in invalid_inputs {
            let result = PaymentStatus::from_string(input);
            assert!(result.is_none(), "Should be None for input: {}", input);
        }
    }

    #[test]
    fn test_payment_status_equality() {
        let paid1 = PaymentStatus::Paid;
        let paid2 = PaymentStatus::Paid;
        let installment = PaymentStatus::Installment;

        assert_eq!(paid1, paid2);
        assert_ne!(paid1, installment);
        assert_ne!(paid2, installment);
    }

    #[test]
    fn test_payment_status_clone() {
        let original = PaymentStatus::Paid;
        let cloned = original.clone();

        assert_eq!(original, cloned);

        let original_installment = PaymentStatus::Installment;
        let cloned_installment = original_installment.clone();

        assert_eq!(original_installment, cloned_installment);
    }

    #[test]
    fn test_payment_status_debug_format() {
        let paid_debug = format!("{:?}", PaymentStatus::Paid);
        let installment_debug = format!("{:?}", PaymentStatus::Installment);

        assert_eq!(paid_debug, "Paid");
        assert_eq!(installment_debug, "Installment");
    }

    #[test]
    fn test_payment_status_serialization() {
        let paid_status = PaymentStatus::Paid;
        let installment_status = PaymentStatus::Installment;

        let paid_json = serde_json::to_string(&paid_status).unwrap();
        let installment_json = serde_json::to_string(&installment_status).unwrap();

        assert_eq!(paid_json, "\"Paid\"");
        assert_eq!(installment_json, "\"Installment\"");

        let deserialized_paid: PaymentStatus = serde_json::from_str(&paid_json).unwrap();
        let deserialized_installment: PaymentStatus = serde_json::from_str(&installment_json).unwrap();

        assert_eq!(deserialized_paid, PaymentStatus::Paid);
        assert_eq!(deserialized_installment, PaymentStatus::Installment);
    }

    #[test]
    fn test_payment_status_roundtrip_conversion() {
        let statuses = vec![PaymentStatus::Paid, PaymentStatus::Installment];

        for status in statuses {
            let status_string = status.to_string();
            let parsed_status = PaymentStatus::from_string(&status_string);
            
            assert!(parsed_status.is_some());
            assert_eq!(parsed_status.unwrap(), status);
        }
    }

    #[test]
    fn test_payment_status_match_pattern() {
        let paid = PaymentStatus::Paid;
        let installment = PaymentStatus::Installment;

        match paid {
            PaymentStatus::Paid => assert!(true),
            PaymentStatus::Installment => panic!("Should not match Installment"),
        }

        match installment {
            PaymentStatus::Paid => panic!("Should not match Paid"),
            PaymentStatus::Installment => assert!(true),
        }
    }    #[test]
    fn test_payment_status_match_coverage_paid() {
        let paid = PaymentStatus::Paid;
        
        let result = match paid {
            PaymentStatus::Paid => "correctly_matched_paid",
            PaymentStatus::Installment => "incorrectly_matched_installment",
        };
        
        assert_eq!(result, "correctly_matched_paid");
        
        assert!(matches!(paid, PaymentStatus::Paid));
        assert!(!matches!(paid, PaymentStatus::Installment));
    }

    #[test]
    fn test_payment_status_match_coverage_installment() {
        let installment = PaymentStatus::Installment;
        
        let result = match installment {
            PaymentStatus::Paid => "incorrectly_matched_paid", 
            PaymentStatus::Installment => "correctly_matched_installment",
        };
        
        assert_eq!(result, "correctly_matched_installment");
        
        assert!(matches!(installment, PaymentStatus::Installment));
        assert!(!matches!(installment, PaymentStatus::Paid));
    }

    #[test]
    fn test_payment_status_exhaustive_matching() {
        let test_cases = vec![
            (PaymentStatus::Paid, "Paid variant"),
            (PaymentStatus::Installment, "Installment variant"),
        ];

        for (status, description) in test_cases {
            let result = match status {
                PaymentStatus::Paid => "matched_paid",
                PaymentStatus::Installment => "matched_installment",
            };

            match status {
                PaymentStatus::Paid => assert_eq!(result, "matched_paid", "{}", description),
                PaymentStatus::Installment => assert_eq!(result, "matched_installment", "{}", description),
            }
        }
    }

    #[test]
    fn test_payment_status_conditional_matching() {
        let statuses = vec![PaymentStatus::Paid, PaymentStatus::Installment];
        
        for status in statuses {
            let is_paid = matches!(status, PaymentStatus::Paid);
            let is_installment = matches!(status, PaymentStatus::Installment);
            
            assert!(is_paid ^ is_installment, "Exactly one should be true");
            
            match status {
                PaymentStatus::Paid => {
                    assert!(is_paid);
                    assert!(!is_installment);
                },
                PaymentStatus::Installment => {
                    assert!(!is_paid);
                    assert!(is_installment);
                },
            }
        }
    }

    #[test]
    fn test_payment_status_defensive_branches() {
        
        let variants = vec![PaymentStatus::Paid, PaymentStatus::Installment];
        
        for variant in variants {
            let paid_match = matches!(variant, PaymentStatus::Paid);
            let installment_match = matches!(variant, PaymentStatus::Installment);
            
            assert!(paid_match ^ installment_match, 
                "Each variant should match exactly one pattern");
            
            match variant {
                PaymentStatus::Paid => {
                    assert!(paid_match, "Paid variant should match Paid pattern");
                    assert!(!installment_match, "Paid variant should not match Installment pattern");
                    
                },
                PaymentStatus::Installment => {
                    assert!(installment_match, "Installment variant should match Installment pattern");
                    assert!(!paid_match, "Installment variant should not match Paid pattern");
                    
                },
            }
        }
    }
}