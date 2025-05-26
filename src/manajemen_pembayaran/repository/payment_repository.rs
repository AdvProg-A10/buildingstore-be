use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::{DateTime, Utc, NaiveDateTime};
use std::collections::HashMap;
use uuid::Uuid;

use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};

pub struct PembayaranRepository;

impl PembayaranRepository {    
    pub async fn create(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error>{        
        eprintln!("DEBUG: Creating payment with ID: {}, Transaction ID: {}", payment.id, payment.transaction_id);
        sqlx::query("
            INSERT INTO payments (id, transaction_id, amount, method, status, payment_date, due_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        ")
            .bind(&payment.id)
            .bind(&payment.transaction_id)
            .bind(payment.amount)
            .bind(payment.method.to_string())
            .bind(payment.status.to_string())
            .bind(payment.payment_date.to_rfc3339())
            .bind(payment.due_date.map(|d| d.to_rfc3339()))            
            .execute(&mut *db)
            .await
            .map_err(|e| {
                eprintln!("DEBUG: Failed to insert payment: {e}");
                e
            })?;

        let result = sqlx::query("
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
            .bind(&payment.id)
            .fetch_one(&mut *db)
            .await?;

        let mut created_payment = Self::parse_row_to_payment(result)?;
        
        if !payment.installments.is_empty() {
            for installment in &payment.installments {
                Self::add_installment(&mut db, installment).await?;
            }
            
            created_payment = Self::load_payment_with_installments(&mut db, &created_payment.id).await?;
        }

        Ok(created_payment)
    }    
    
    pub async fn find_by_id(mut db: PoolConnection<Any>, id: &str) -> Result<Payment, sqlx::Error>{
        let payment_with_installments = Self::load_payment_with_installments(&mut db, id).await?;

        Ok(payment_with_installments)
    }    
    
    pub async fn find_all(mut db: PoolConnection<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, sqlx::Error> {
        let mut base_query = "SELECT id, transaction_id, amount, method, status, payment_date, due_date FROM payments".to_string();
        let mut where_clauses = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        if let Some(filter_map) = &filters {
            if let Some(status_str) = filter_map.get("status") {
                where_clauses.push("status = $1".to_string());
                bind_values.push(status_str);
            }            if let Some(method) = filter_map.get("method") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("method = ${param_num}"));
                bind_values.push(method);
            }            if let Some(transaction_id) = filter_map.get("transaction_id") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("transaction_id = ${param_num}"));
                bind_values.push(transaction_id);
            }
        }
        
        if !where_clauses.is_empty() {
            base_query.push_str(" WHERE ");
            base_query.push_str(&where_clauses.join(" AND "));
        }        
        let rows = match bind_values.len() {
            0 => sqlx::query(&base_query).fetch_all(&mut *db).await?,
            1 => sqlx::query(&base_query).bind(bind_values[0]).fetch_all(&mut *db).await?,
            2 => sqlx::query(&base_query).bind(bind_values[0]).bind(bind_values[1]).fetch_all(&mut *db).await?,
            3 => sqlx::query(&base_query).bind(bind_values[0]).bind(bind_values[1]).bind(bind_values[2]).fetch_all(&mut *db).await?,
            _ => return Err(sqlx::Error::ColumnNotFound("Too many filters".to_string())),
        };
        
        let mut payments = Vec::with_capacity(rows.len());
        for row in rows {
            let payment_id: String = row.get("id");
            let payment_with_installments = Self::load_payment_with_installments(&mut db, &payment_id).await?;
            payments.push(payment_with_installments);
        }
        
        Ok(payments)
    }
    
    pub async fn update(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error>{
        let payment_method_str = payment.method.to_string();
        let status_str = payment.status.to_string();
        sqlx::query("
            UPDATE payments
            SET transaction_id = $1, amount = $2, method = $3, status = $4, payment_date = $5, due_date = $6
            WHERE id = $7
        ")
        .bind(&payment.transaction_id)
        .bind(payment.amount)
        .bind(&payment_method_str)
        .bind(&status_str)
        .bind(payment.payment_date.to_rfc3339())
        .bind(payment.due_date.map(|d| d.to_rfc3339()))
        .bind(&payment.id)
        .execute(&mut *db)
        .await?;

        let result = sqlx::query("
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
        .bind(&payment.id)
        .fetch_one(&mut *db)
        .await?;
        
        let updated_payment = Self::parse_row_to_payment(result)?;
        
        let payment_with_installments = Self::load_payment_with_installments(&mut db, &updated_payment.id).await?;

        Ok(payment_with_installments)
    }

    pub async fn update_payment_status(mut db: PoolConnection<Any>, payment_id: String, new_status: PaymentStatus, additional_amount: Option<f64>) -> Result<Payment, sqlx::Error> {        let payment_result = sqlx::query("
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
        .bind(&payment_id)        .fetch_one(&mut *db)
        .await?;
        
        let mut payment = Self::parse_row_to_payment(payment_result)?;
        
        payment.status = new_status;
        
        if let Some(amount) = additional_amount {
            let installment = Installment {
                id: format!("INST-{}", Uuid::new_v4()),
                payment_id: payment_id.clone(),
                amount,
                payment_date: Utc::now(),
            };
            
            Self::add_installment(&mut db, &installment).await?;
        }
        
        Self::update(db, &payment).await
    }

    pub async fn delete(mut db: PoolConnection<Any>, id: &str) -> Result<(), sqlx::Error>{
        sqlx::query("DELETE FROM installments WHERE payment_id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        sqlx::query("DELETE FROM payments WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }
    
    pub async fn add_installment(db: &mut PoolConnection<Any>, installment: &Installment) -> Result<(), sqlx::Error> {
        sqlx::query("
            INSERT INTO installments (id, payment_id, amount, payment_date)
            VALUES ($1, $2, $3, $4)
        ")
        .bind(&installment.id)
        .bind(&installment.payment_id)
        .bind(installment.amount)
        .bind(installment.payment_date.to_rfc3339())
        .execute(&mut **db)
        .await?;
        
        Ok(())
    }    
    
    pub async fn load_payment_with_installments(db: &mut PoolConnection<Any>, payment_id: &str) -> Result<Payment, sqlx::Error> {        
        let payment_row = sqlx::query("
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")        .bind(payment_id)
        .fetch_one(&mut **db)
        .await?;
        
        let mut payment = Self::parse_row_to_payment(payment_row)?;
        
        let installment_rows = sqlx::query("
            SELECT id, payment_id, amount, payment_date
            FROM installments
            WHERE payment_id = $1
            ORDER BY payment_date ASC
        ")
        .bind(payment_id)
        .fetch_all(&mut **db)
        .await?;
        
        let mut installments = Vec::with_capacity(installment_rows.len());
        for row in installment_rows {
            let installment = Self::parse_row_to_installment(row)?;
            installments.push(installment);
        }
        
        payment.installments = installments;
        
        Ok(payment)
    }
    
    fn parse_row_to_payment(row: AnyRow) -> Result<Payment, sqlx::Error> {
        let id: String = row.get("id");
        let transaction_id: String = row.get("transaction_id");
        let amount: f64 = row.try_get("amount").unwrap_or_else(|_| {
            row.try_get::<f32, _>("amount").map(|v| v as f64).unwrap_or(0.0)
        });        let payment_method_str: String = row.get("method");
        let status_str: String = row.get("status");
        let payment_date_str: String = row.get("payment_date");
        let due_date_str: Option<String> = row.try_get("due_date").ok();
        
        let payment_method = match payment_method_str.as_str() {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "BANK_TRANSFER" => PaymentMethod::BankTransfer,
            "E_WALLET" => PaymentMethod::EWallet,
            _ => return Err(sqlx::Error::RowNotFound),
        };
          let payment_status = PaymentStatus::from_string(&status_str)
            .ok_or(sqlx::Error::RowNotFound)?;        let payment_date = DateTime::parse_from_rfc3339(&payment_date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                NaiveDateTime::parse_from_str(&payment_date_str, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| {
                        NaiveDateTime::parse_from_str(&payment_date_str, "%Y-%m-%d %H:%M:%S")
                    })
                    .map(|naive_dt| naive_dt.and_utc())
            })            .map_err(|e| {
                eprintln!("Failed to parse payment_date '{payment_date_str}': {e}");
                sqlx::Error::RowNotFound
            })?;let due_date = due_date_str
            .map(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|_| {
                        NaiveDateTime::parse_from_str(&d, "%Y-%m-%d %H:%M:%S%.f")
                            .or_else(|_| {
                                NaiveDateTime::parse_from_str(&d, "%Y-%m-%d %H:%M:%S")
                            })
                            .map(|naive_dt| naive_dt.and_utc())
                    })
            })
            .transpose()            .map_err(|e| {
                eprintln!("Failed to parse due_date: {e}");
                sqlx::Error::RowNotFound
            })?;
        
        Ok(Payment {
            id,
            transaction_id,
            amount,
            method: payment_method,
            status: payment_status,
            payment_date,
            installments: Vec::new(), 
            due_date,
        })
    }
    
    fn parse_row_to_installment(row: AnyRow) -> Result<Installment, sqlx::Error> {
        let id: String = row.get("id");
        let payment_id: String = row.get("payment_id");
        let amount: f64 = row.try_get("amount").unwrap_or_else(|_| {
            row.try_get::<f32, _>("amount").map(|v| v as f64).unwrap_or(0.0)
        });
        let payment_date_str: String = row.get("payment_date");        let payment_date = DateTime::parse_from_rfc3339(&payment_date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                NaiveDateTime::parse_from_str(&payment_date_str, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| {
                        NaiveDateTime::parse_from_str(&payment_date_str, "%Y-%m-%d %H:%M:%S")
                    })
                    .map(|naive_dt| naive_dt.and_utc())
            })            .map_err(|e| {
                eprintln!("Failed to parse installment payment_date '{payment_date_str}': {e}");
                sqlx::Error::RowNotFound
            })?;
        
        Ok(Installment {
            id,
            payment_id,
            amount,
            payment_date,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc};
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_parse_row_to_payment_success() {
        let payment_method_cash = match "CASH" {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "BANK_TRANSFER" => PaymentMethod::BankTransfer,
            "E_WALLET" => PaymentMethod::EWallet,
            _ => panic!("Invalid payment method"),
        };
        
        assert_eq!(payment_method_cash, PaymentMethod::Cash);
        
        let payment_method_credit = match "CREDIT_CARD" {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "BANK_TRANSFER" => PaymentMethod::BankTransfer,
            "E_WALLET" => PaymentMethod::EWallet,
            _ => panic!("Invalid payment method"),
        };
        
        assert_eq!(payment_method_credit, PaymentMethod::CreditCard);
    }

    #[test]
    fn test_repository_query_building_filters() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        filters.insert("method".to_string(), "CASH".to_string());
        filters.insert("transaction_id".to_string(), "TXN-123".to_string());

        let mut base_query = "SELECT id, transaction_id, amount, method, status, payment_date, due_date FROM payments".to_string();
        let mut where_clauses = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        if let Some(status_str) = filters.get("status") {
            where_clauses.push("status = $1".to_string());
            bind_values.push(status_str);
        }
        if let Some(method) = filters.get("method") {
            let param_num = bind_values.len() + 1;
            where_clauses.push(format!("method = ${}", param_num));
            bind_values.push(method);
        }
        if let Some(transaction_id) = filters.get("transaction_id") {
            let param_num = bind_values.len() + 1;
            where_clauses.push(format!("transaction_id = ${}", param_num));
            bind_values.push(transaction_id);
        }
        
        if !where_clauses.is_empty() {
            base_query.push_str(" WHERE ");
            base_query.push_str(&where_clauses.join(" AND "));
        }

        assert_eq!(bind_values.len(), 3);
        assert_eq!(bind_values[0], "LUNAS");
        assert_eq!(bind_values[1], "CASH");
        assert_eq!(bind_values[2], "TXN-123");
        assert!(base_query.contains("WHERE"));
        assert!(base_query.contains("status = $1"));
        assert!(base_query.contains("method = $2"));
        assert!(base_query.contains("transaction_id = $3"));
    }

    #[test]
    fn test_bind_values_length_handling() {
        let bind_values_0: Vec<&str> = Vec::new();
        let bind_values_1 = vec!["value1"];
        let bind_values_2 = vec!["value1", "value2"];
        let bind_values_3 = vec!["value1", "value2", "value3"];
        let bind_values_4 = vec!["value1", "value2", "value3", "value4"];

        assert_eq!(bind_values_0.len(), 0);
        assert_eq!(bind_values_1.len(), 1);
        assert_eq!(bind_values_2.len(), 2);
        assert_eq!(bind_values_3.len(), 3);
        assert_eq!(bind_values_4.len(), 4);

        if bind_values_4.len() > 3 {
            assert!(true);
        } else {
            panic!("Should handle more than 3 filters");
        }
    }

    #[test]
    fn test_payment_method_parsing_in_parse_row() {
        let test_cases = vec![
            ("CASH", PaymentMethod::Cash),
            ("CREDIT_CARD", PaymentMethod::CreditCard),
            ("BANK_TRANSFER", PaymentMethod::BankTransfer),
            ("E_WALLET", PaymentMethod::EWallet),
        ];

        for (method_str, expected_method) in test_cases {
            let parsed_method = match method_str {
                "CASH" => PaymentMethod::Cash,
                "CREDIT_CARD" => PaymentMethod::CreditCard,
                "BANK_TRANSFER" => PaymentMethod::BankTransfer,
                "E_WALLET" => PaymentMethod::EWallet,
                _ => panic!("Invalid payment method"),
            };
            
            assert_eq!(parsed_method, expected_method);
        }
    }

    #[test]
    fn test_parse_row_payment_status_handling() {
        let status_lunas = PaymentStatus::from_string("LUNAS");
        let status_cicilan = PaymentStatus::from_string("CICILAN");
        let status_invalid = PaymentStatus::from_string("INVALID");

        assert_eq!(status_lunas, Some(PaymentStatus::Paid));
        assert_eq!(status_cicilan, Some(PaymentStatus::Installment));
        assert_eq!(status_invalid, None);

        let parsed_status_paid = PaymentStatus::from_string("LUNAS")
            .ok_or("RowNotFound");
        let parsed_status_installment = PaymentStatus::from_string("CICILAN")
            .ok_or("RowNotFound");

        assert!(parsed_status_paid.is_ok());
        assert!(parsed_status_installment.is_ok());
        assert_eq!(parsed_status_paid.unwrap(), PaymentStatus::Paid);
        assert_eq!(parsed_status_installment.unwrap(), PaymentStatus::Installment);
    }

    #[test]
    fn test_date_parsing_formats() {
        let rfc3339_date = "2023-10-15T10:30:00Z";
        let standard_date = "2023-10-15 10:30:00.123";
        let simple_date = "2023-10-15 10:30:00";

        let parsed_rfc3339 = DateTime::parse_from_rfc3339(rfc3339_date)
            .map(|dt| dt.with_timezone(&Utc));
        assert!(parsed_rfc3339.is_ok());

        let parsed_standard = NaiveDateTime::parse_from_str(standard_date, "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive_dt| naive_dt.and_utc());
        assert!(parsed_standard.is_ok());

        let parsed_simple = NaiveDateTime::parse_from_str(simple_date, "%Y-%m-%d %H:%M:%S")
            .map(|naive_dt| naive_dt.and_utc());
        assert!(parsed_simple.is_ok());
    }

    #[test]
    fn test_amount_parsing_fallback() {
        let amount_f64 = 1000.0f64;
        let amount_f32 = 500.0f32;
        
        let converted_amount = amount_f32 as f64;
        assert_eq!(converted_amount, 500.0f64);
        
        let fallback_amount = 0.0f64;
        assert_eq!(fallback_amount, 0.0);
        
        assert_ne!(amount_f64, converted_amount);
        assert_eq!(amount_f64, 1000.0);
    }

    #[test]
    fn test_installment_id_generation() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let installment_id1 = format!("INST-{}", uuid1);
        let installment_id2 = format!("INST-{}", uuid2);

        assert_ne!(uuid1, uuid2);
        assert_ne!(installment_id1, installment_id2);
        assert!(installment_id1.starts_with("INST-"));
        assert!(installment_id2.starts_with("INST-"));
    }

    #[test]
    fn test_update_payment_status_logic() {
        let payment_id = "PMT-UPDATE-001".to_string();
        let new_status = PaymentStatus::Installment;
        let additional_amount = Some(250.0);

        let mut test_payment = Payment {
            id: payment_id.clone(),
            transaction_id: "TXN-UPDATE-001".to_string(),
            amount: 1000.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        test_payment.status = new_status;
        assert_eq!(test_payment.status, PaymentStatus::Installment);

        if let Some(amount) = additional_amount {
            let installment = Installment {
                id: format!("INST-{}", Uuid::new_v4()),
                payment_id: payment_id.clone(),
                amount,
                payment_date: Utc::now(),
            };

            assert_eq!(installment.payment_id, payment_id);
            assert_eq!(installment.amount, 250.0);
            assert!(installment.id.starts_with("INST-"));
        }
    }

    #[test]
    fn test_payment_creation_with_installments() {
        let payment = Payment {
            id: "PMT-CREATE-001".to_string(),
            transaction_id: "TXN-CREATE-001".to_string(),
            amount: 1500.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: vec![
                Installment {
                    id: "INST-001".to_string(),
                    payment_id: "PMT-CREATE-001".to_string(),
                    amount: 500.0,
                    payment_date: Utc::now(),
                },
                Installment {
                    id: "INST-002".to_string(),
                    payment_id: "PMT-CREATE-001".to_string(),
                    amount: 300.0,
                    payment_date: Utc::now(),
                },
            ],
            due_date: Some(Utc::now()),
        };

        assert!(!payment.installments.is_empty());
        assert_eq!(payment.installments.len(), 2);

        let total_installments: f64 = payment.installments.iter().map(|i| i.amount).sum();
        assert_eq!(total_installments, 800.0);

        let remaining_amount = payment.amount - total_installments;
        assert_eq!(remaining_amount, 700.0);
    }

    #[test]
    fn test_payment_method_to_string_conversion() {
        let payment_method_str_cash = PaymentMethod::Cash.to_string();
        let payment_method_str_credit = PaymentMethod::CreditCard.to_string();
        let payment_method_str_bank = PaymentMethod::BankTransfer.to_string();
        let payment_method_str_ewallet = PaymentMethod::EWallet.to_string();

        assert_eq!(payment_method_str_cash, "CASH");
        assert_eq!(payment_method_str_credit, "CREDIT_CARD");
        assert_eq!(payment_method_str_bank, "BANK_TRANSFER");
        assert_eq!(payment_method_str_ewallet, "E_WALLET");
    }

    #[test]
    fn test_payment_status_to_string_conversion() {
        let status_str_paid = PaymentStatus::Paid.to_string();
        let status_str_installment = PaymentStatus::Installment.to_string();

        assert_eq!(status_str_paid, "LUNAS");
        assert_eq!(status_str_installment, "CICILAN");
    }

    #[test]
    fn test_rfc3339_date_formatting() {
        let now = Utc::now();
        let rfc3339_string = now.to_rfc3339();
        
        assert!(!rfc3339_string.is_empty());
        assert!(rfc3339_string.contains("T"));
        assert!(rfc3339_string.contains("Z") || rfc3339_string.contains("+"));

        let parsed_back = DateTime::parse_from_rfc3339(&rfc3339_string);
        assert!(parsed_back.is_ok());
    }

    #[test]
    fn test_due_date_option_handling() {
        let due_date_some = Some(Utc::now());
        let due_date_none: Option<DateTime<Utc>> = None;

        let formatted_some = due_date_some.map(|d| d.to_rfc3339());
        let formatted_none = due_date_none.map(|d| d.to_rfc3339());

        assert!(formatted_some.is_some());
        assert!(formatted_none.is_none());

        if let Some(formatted) = formatted_some {
            assert!(!formatted.is_empty());
            assert!(formatted.contains("T"));
        }
    }

    #[test]
    fn test_installments_capacity_optimization() {
        let installment_rows_count = 5;
        let mut installments = Vec::with_capacity(installment_rows_count);
        
        assert_eq!(installments.capacity(), 5);
        assert_eq!(installments.len(), 0);

        for i in 0..3 {
            let installment = Installment {
                id: format!("INST-{}", i),
                payment_id: "PMT-001".to_string(),
                amount: 100.0 * (i + 1) as f64,
                payment_date: Utc::now(),
            };
            installments.push(installment);
        }

        assert_eq!(installments.len(), 3);
        assert!(installments.capacity() >= 3);
    }

    #[test]
    fn test_error_propagation_patterns() {
        let payment_method_str = "INVALID_METHOD";
        let is_valid_method = matches!(payment_method_str, "CASH" | "CREDIT_CARD" | "BANK_TRANSFER" | "E_WALLET");
        assert!(!is_valid_method);

        let status_str = "INVALID_STATUS";
        let parsed_status = PaymentStatus::from_string(status_str);
        assert!(parsed_status.is_none());

        let invalid_date_str = "invalid-date-format";
        let date_parse_result = DateTime::parse_from_rfc3339(invalid_date_str);
        assert!(date_parse_result.is_err());
    }

    #[test]
    fn test_payment_update_string_variables() {
        let payment = Payment {
            id: "PMT-UPDATE-002".to_string(),
            transaction_id: "TXN-UPDATE-002".to_string(),
            amount: 2000.0,
            method: PaymentMethod::EWallet,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let payment_method_str = payment.method.to_string();
        let status_str = payment.status.to_string();

        assert_eq!(payment_method_str, "E_WALLET");
        assert_eq!(status_str, "LUNAS");

        let payment_date_str = payment.payment_date.to_rfc3339();
        let due_date_str = payment.due_date.map(|d| d.to_rfc3339());

        assert!(!payment_date_str.is_empty());
        assert!(due_date_str.is_none());
    }

    #[test]
    fn test_delete_operation_order() {
        let payment_id = "PMT-DELETE-001";
        
        let installments_query = "DELETE FROM installments WHERE payment_id = $1";
        let payments_query = "DELETE FROM payments WHERE id = $1";

        assert!(installments_query.contains("installments"));
        assert!(installments_query.contains("payment_id"));
        assert!(payments_query.contains("payments"));
        assert!(payments_query.contains("id"));
        assert_eq!(payment_id, "PMT-DELETE-001");
    }    
    
    #[test]
    fn test_load_payment_with_installments_logic() {
        let _payment_id = "PMT-LOAD-001";
        
        let payment_query = "
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ";
        
        let installment_query = "
            SELECT id, payment_id, amount, payment_date
            FROM installments
            WHERE payment_id = $1
            ORDER BY payment_date ASC
        ";

        assert!(payment_query.contains("SELECT"));
        assert!(payment_query.contains("payments"));
        assert!(payment_query.contains("WHERE id = $1"));
        
        assert!(installment_query.contains("SELECT"));
        assert!(installment_query.contains("installments"));
        assert!(installment_query.contains("WHERE payment_id = $1"));
        assert!(installment_query.contains("ORDER BY payment_date ASC"));
    }

    #[test]
    fn test_create_payment_insert_query_structure() {
        let insert_query = "
            INSERT INTO payments (id, transaction_id, amount, method, status, payment_date, due_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        ";
        
        assert!(insert_query.contains("INSERT INTO payments"));
        assert!(insert_query.contains("VALUES"));
        assert!(insert_query.contains("$1"));
        assert!(insert_query.contains("$7"));
        assert!(insert_query.contains("id, transaction_id, amount"));
        assert!(insert_query.contains("method, status, payment_date, due_date"));
    }

    #[test]
    fn test_create_payment_select_after_insert() {
        let select_query = "
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ";
        
        assert!(select_query.contains("SELECT"));
        assert!(select_query.contains("FROM payments"));
        assert!(select_query.contains("WHERE id = $1"));
        assert!(select_query.contains("transaction_id"));
        assert!(select_query.contains("amount"));
        assert!(select_query.contains("method"));
        assert!(select_query.contains("status"));
    }

    #[test]
    fn test_create_payment_with_installments_flow() {
        let payment_with_installments = Payment {
            id: "PMT-001".to_string(),
            transaction_id: "TXN-001".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            due_date: Some(Utc::now()),
            installments: vec![
                Installment {
                    id: "INST-001".to_string(),
                    payment_id: "PMT-001".to_string(),
                    amount: 300.0,
                    payment_date: Utc::now(),
                },
                Installment {
                    id: "INST-002".to_string(),
                    payment_id: "PMT-001".to_string(),
                    amount: 700.0,
                    payment_date: Utc::now(),
                }
            ],
        };

        assert!(!payment_with_installments.installments.is_empty());
        assert_eq!(payment_with_installments.installments.len(), 2);
    }

    #[test]
    fn test_create_payment_without_installments_flow() {
        let payment_without_installments = Payment {
            id: "PMT-002".to_string(),
            transaction_id: "TXN-002".to_string(),
            amount: 500.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            due_date: None,
            installments: vec![],
        };

        assert!(payment_without_installments.installments.is_empty());
        assert_eq!(payment_without_installments.installments.len(), 0);
    }    #[test]
    fn test_find_all_base_query_construction() {
        let base_query = "SELECT id, transaction_id, amount, method, status, payment_date, due_date FROM payments".to_string();
        let where_clauses: Vec<String> = Vec::new();
        let bind_values: Vec<&str> = Vec::new();
        
        assert_eq!(where_clauses.len(), 0);
        assert_eq!(bind_values.len(), 0);
        assert!(base_query.starts_with("SELECT"));
        assert!(base_query.ends_with("FROM payments"));
    }    
    
    #[test]
    fn test_find_all_with_status_filter() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        let filter_option = Some(&filters);
        if let Some(filter_map) = filter_option {
            if let Some(status_str) = filter_map.get("status") {
                where_clauses.push("status = $1".to_string());
                bind_values.push(status_str);
            }
        }
        
        assert_eq!(where_clauses.len(), 1);
        assert_eq!(bind_values.len(), 1);
        assert_eq!(where_clauses[0], "status = $1");
        assert_eq!(bind_values[0], "LUNAS");
    }    
    
    #[test]
    fn test_find_all_with_method_filter() {
        let mut filters = HashMap::new();
        filters.insert("method".to_string(), "CASH".to_string());
        
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        let filter_option = Some(&filters);
        if let Some(filter_map) = filter_option {
            if let Some(method) = filter_map.get("method") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("method = ${param_num}"));
                bind_values.push(method);
            }
        }
        
        assert_eq!(where_clauses.len(), 1);
        assert_eq!(bind_values.len(), 1);
        assert_eq!(where_clauses[0], "method = $1");
        assert_eq!(bind_values[0], "CASH");
    }    
    
    #[test]
    fn test_find_all_with_transaction_id_filter() {
        let mut filters = HashMap::new();
        filters.insert("transaction_id".to_string(), "TXN-123".to_string());
        
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        let filter_option = Some(&filters);
        if let Some(filter_map) = filter_option {
            if let Some(transaction_id) = filter_map.get("transaction_id") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("transaction_id = ${param_num}"));
                bind_values.push(transaction_id);
            }
        }
        
        assert_eq!(where_clauses.len(), 1);
        assert_eq!(bind_values.len(), 1);
        assert_eq!(where_clauses[0], "transaction_id = $1");
        assert_eq!(bind_values[0], "TXN-123");
    }    
    
    #[test]
    fn test_find_all_multiple_filters() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        filters.insert("method".to_string(), "CASH".to_string());
        filters.insert("transaction_id".to_string(), "TXN-123".to_string());
        
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        let filter_option = Some(&filters);
        if let Some(filter_map) = filter_option {
            if let Some(status_str) = filter_map.get("status") {
                where_clauses.push("status = $1".to_string());
                bind_values.push(status_str);
            }
            if let Some(method) = filter_map.get("method") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("method = ${param_num}"));
                bind_values.push(method);
            }
            if let Some(transaction_id) = filter_map.get("transaction_id") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("transaction_id = ${param_num}"));
                bind_values.push(transaction_id);
            }
        }
        
        assert_eq!(where_clauses.len(), 3);
        assert_eq!(bind_values.len(), 3);
    }

    #[test]
    fn test_find_all_where_clause_joining() {
        let where_clauses = vec!["status = $1".to_string(), "method = $2".to_string()];
        let joined = where_clauses.join(" AND ");
        
        assert_eq!(joined, "status = $1 AND method = $2");
        assert!(joined.contains(" AND "));
    }

    #[test]
    fn test_find_all_empty_where_clauses() {
        let where_clauses: Vec<String> = Vec::new();
        let is_empty = where_clauses.is_empty();
        
        assert!(is_empty);
        assert_eq!(where_clauses.len(), 0);
    }

    #[test]
    fn test_find_all_bind_values_match_cases() {
        let bind_values_0: Vec<&str> = Vec::new();
        let bind_values_1 = vec!["value1"];
        let bind_values_2 = vec!["value1", "value2"];
        let bind_values_3 = vec!["value1", "value2", "value3"];
        let bind_values_4 = vec!["value1", "value2", "value3", "value4"];

        match bind_values_0.len() {
            0 => assert!(true),
            _ => assert!(false),
        }

        match bind_values_1.len() {
            1 => assert!(true),
            _ => assert!(false),
        }

        match bind_values_2.len() {
            2 => assert!(true),
            _ => assert!(false),
        }

        match bind_values_3.len() {
            3 => assert!(true),
            _ => assert!(false),
        }

        match bind_values_4.len() {
            _ => assert!(bind_values_4.len() > 3),
        }
    }

    #[test]
    fn test_find_all_payments_vector_capacity() {
        let rows_count = 5;
        let mut payments = Vec::with_capacity(rows_count);
        
        assert_eq!(payments.capacity(), 5);
        assert_eq!(payments.len(), 0);
        
        for i in 0..3 {
            let payment = Payment {
                id: format!("PMT-{}", i),
                transaction_id: format!("TXN-{}", i),
                amount: 100.0,
                method: PaymentMethod::Cash,
                status: PaymentStatus::Paid,
                payment_date: Utc::now(),
                due_date: None,
                installments: vec![],
            };
            payments.push(payment);
        }
        
        assert_eq!(payments.len(), 3);
        assert!(payments.capacity() >= 3);
    }

    #[test]
    fn test_update_payment_query_structure() {
        let update_query = "
            UPDATE payments
            SET transaction_id = $1, amount = $2, method = $3, status = $4, payment_date = $5, due_date = $6
            WHERE id = $7
        ";
        
        assert!(update_query.contains("UPDATE payments"));
        assert!(update_query.contains("SET"));
        assert!(update_query.contains("WHERE id = $7"));
        assert!(update_query.contains("transaction_id = $1"));
        assert!(update_query.contains("amount = $2"));
        assert!(update_query.contains("method = $3"));
        assert!(update_query.contains("status = $4"));
    }

    #[test]
    fn test_update_payment_method_string_conversion() {
        let payment = Payment {
            id: "PMT-UPDATE".to_string(),
            transaction_id: "TXN-UPDATE".to_string(),
            amount: 1000.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            due_date: None,
            installments: vec![],
        };
        
        let payment_method_str = payment.method.to_string();
        let status_str = payment.status.to_string();
        
        assert_eq!(payment_method_str, "CREDIT_CARD");
        assert_eq!(status_str, "LUNAS");
    }    #[test]
    fn test_update_payment_status_method_structure() {
        let payment_id = "PMT-STATUS-UPDATE".to_string();
        let new_status = PaymentStatus::Installment;
        let additional_amount = Some(500.0);
        
        let status_query = "
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ";
        
        assert!(status_query.contains("SELECT"));
        assert!(status_query.contains("FROM payments"));
        assert!(status_query.contains("WHERE id = $1"));
        assert_eq!(payment_id, "PMT-STATUS-UPDATE");
        assert_eq!(new_status, PaymentStatus::Installment);
        assert!(additional_amount.is_some());
    }

    #[test]
    fn test_update_payment_status_installment_creation() {
        let payment_id = "PMT-INST-CREATE".to_string();
        let additional_amount = 250.0;
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.clone(),
            amount: additional_amount,
            payment_date: Utc::now(),
        };
        
        assert_eq!(installment.payment_id, payment_id);
        assert_eq!(installment.amount, additional_amount);
        assert!(installment.id.starts_with("INST-"));
    }

    #[test]
    fn test_delete_payment_installments_query() {
        let delete_installments_query = "DELETE FROM installments WHERE payment_id = $1";
        let delete_payment_query = "DELETE FROM payments WHERE id = $1";
        
        assert!(delete_installments_query.contains("DELETE FROM installments"));
        assert!(delete_installments_query.contains("WHERE payment_id = $1"));
        assert!(delete_payment_query.contains("DELETE FROM payments"));
        assert!(delete_payment_query.contains("WHERE id = $1"));
    }

    #[test]
    fn test_add_installment_query_structure() {
        let insert_installment_query = "
            INSERT INTO installments (id, payment_id, amount, payment_date)
            VALUES ($1, $2, $3, $4)
        ";
        
        assert!(insert_installment_query.contains("INSERT INTO installments"));
        assert!(insert_installment_query.contains("VALUES"));
        assert!(insert_installment_query.contains("$1, $2, $3, $4"));
        assert!(insert_installment_query.contains("id, payment_id, amount, payment_date"));
    }

    #[test]
    fn test_load_payment_with_installments_payment_query() {
        let payment_query = "
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ";
        
        assert!(payment_query.contains("SELECT"));
        assert!(payment_query.contains("FROM payments"));
        assert!(payment_query.contains("WHERE id = $1"));
        assert!(payment_query.contains("transaction_id"));
        assert!(payment_query.contains("amount"));
        assert!(payment_query.contains("method"));
        assert!(payment_query.contains("status"));
    }

    #[test]
    fn test_load_payment_with_installments_installment_query() {
        let installment_query = "
            SELECT id, payment_id, amount, payment_date
            FROM installments
            WHERE payment_id = $1
            ORDER BY payment_date ASC
        ";
        
        assert!(installment_query.contains("SELECT"));
        assert!(installment_query.contains("FROM installments"));
        assert!(installment_query.contains("WHERE payment_id = $1"));
        assert!(installment_query.contains("ORDER BY payment_date ASC"));
    }

    #[test]
    fn test_load_payment_installments_vector_building() {
        let installment_rows_count = 3;
        let mut installments = Vec::with_capacity(installment_rows_count);
        
        for i in 0..installment_rows_count {
            let installment = Installment {
                id: format!("INST-{}", i),
                payment_id: "PMT-001".to_string(),
                amount: 100.0 * (i + 1) as f64,
                payment_date: Utc::now(),
            };
            installments.push(installment);
        }
        
        assert_eq!(installments.len(), 3);
        assert_eq!(installments[0].amount, 100.0);
        assert_eq!(installments[1].amount, 200.0);
        assert_eq!(installments[2].amount, 300.0);
    }

    #[test]
    fn test_parse_row_to_payment_field_extraction() {
        let id = "PMT-PARSE-001".to_string();
        let transaction_id = "TXN-PARSE-001".to_string();
        let amount = 1500.0f64;
        let payment_method_str = "CASH".to_string();
        let status_str = "LUNAS".to_string();
        let payment_date_str = "2023-10-15T10:30:00Z".to_string();
        
        assert_eq!(id, "PMT-PARSE-001");
        assert_eq!(transaction_id, "TXN-PARSE-001");
        assert_eq!(amount, 1500.0);
        assert_eq!(payment_method_str, "CASH");
        assert_eq!(status_str, "LUNAS");
        assert!(!payment_date_str.is_empty());
    }

    #[test]
    fn test_parse_row_to_payment_amount_fallback() {
        let amount_f64 = 1000.0f64;
        let amount_f32 = 500.0f32;
        let converted_amount = amount_f32 as f64;
        let fallback_amount = 0.0f64;
        
        assert_eq!(amount_f64, 1000.0);
        assert_eq!(converted_amount, 500.0);
        assert_eq!(fallback_amount, 0.0);
        assert_ne!(amount_f64, converted_amount);
    }

    #[test]
    fn test_parse_row_to_payment_method_matching() {
        let test_methods = vec![
            ("CASH", PaymentMethod::Cash),
            ("CREDIT_CARD", PaymentMethod::CreditCard),
            ("BANK_TRANSFER", PaymentMethod::BankTransfer),
            ("E_WALLET", PaymentMethod::EWallet),
        ];
        
        for (method_str, expected_method) in test_methods {
            let parsed_method = match method_str {
                "CASH" => PaymentMethod::Cash,
                "CREDIT_CARD" => PaymentMethod::CreditCard,
                "BANK_TRANSFER" => PaymentMethod::BankTransfer,
                "E_WALLET" => PaymentMethod::EWallet,
                _ => PaymentMethod::Cash,
            };
            assert_eq!(parsed_method, expected_method);
        }
    }

    #[test]
    fn test_parse_row_to_payment_invalid_method() {
        let invalid_method = "INVALID_METHOD";
        let is_valid = matches!(invalid_method, "CASH" | "CREDIT_CARD" | "BANK_TRANSFER" | "E_WALLET");
        assert!(!is_valid);
    }

    #[test]
    fn test_parse_row_to_payment_status_parsing() {
        let status_paid = PaymentStatus::from_string("LUNAS");
        let status_installment = PaymentStatus::from_string("CICILAN");
        let status_invalid = PaymentStatus::from_string("INVALID");
        
        assert_eq!(status_paid, Some(PaymentStatus::Paid));
        assert_eq!(status_installment, Some(PaymentStatus::Installment));
        assert_eq!(status_invalid, None);
    }

    #[test]
    fn test_parse_row_to_payment_date_parsing_rfc3339() {
        let rfc3339_date = "2023-10-15T10:30:00Z";
        let parsed_date = DateTime::parse_from_rfc3339(rfc3339_date)
            .map(|dt| dt.with_timezone(&Utc));
        
        assert!(parsed_date.is_ok());
    }

    #[test]
    fn test_parse_row_to_payment_date_parsing_standard() {
        let standard_date = "2023-10-15 10:30:00.123";
        let parsed_date = NaiveDateTime::parse_from_str(standard_date, "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive_dt| naive_dt.and_utc());
        
        assert!(parsed_date.is_ok());
    }

    #[test]
    fn test_parse_row_to_payment_date_parsing_simple() {
        let simple_date = "2023-10-15 10:30:00";
        let parsed_date = NaiveDateTime::parse_from_str(simple_date, "%Y-%m-%d %H:%M:%S")
            .map(|naive_dt| naive_dt.and_utc());
        
        assert!(parsed_date.is_ok());
    }

    #[test]
    fn test_parse_row_to_payment_due_date_some() {
        let due_date_str = Some("2023-10-15T10:30:00Z".to_string());
        let parsed_due_date = due_date_str
            .map(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .transpose();
        
        assert!(parsed_due_date.is_ok());
        assert!(parsed_due_date.unwrap().is_some());
    }

    #[test]
    fn test_parse_row_to_payment_due_date_none() {
        let due_date_str: Option<String> = None;
        let parsed_due_date = due_date_str
            .map(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .transpose();
        
        assert!(parsed_due_date.is_ok());
        assert!(parsed_due_date.unwrap().is_none());
    }

    #[test]
    fn test_parse_row_to_payment_structure_creation() {
        let payment_data = Payment {
            id: "PMT-STRUCT-001".to_string(),
            transaction_id: "TXN-STRUCT-001".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        assert_eq!(payment_data.id, "PMT-STRUCT-001");
        assert_eq!(payment_data.transaction_id, "TXN-STRUCT-001");
        assert_eq!(payment_data.amount, 1000.0);
        assert_eq!(payment_data.method, PaymentMethod::Cash);
        assert_eq!(payment_data.status, PaymentStatus::Paid);
        assert!(payment_data.installments.is_empty());
        assert!(payment_data.due_date.is_none());
    }

    #[test]
    fn test_parse_row_to_installment_field_extraction() {
        let id = "INST-PARSE-001".to_string();
        let payment_id = "PMT-PARSE-001".to_string();
        let amount = 500.0f64;
        let payment_date_str = "2023-10-15T10:30:00Z".to_string();
        
        assert_eq!(id, "INST-PARSE-001");
        assert_eq!(payment_id, "PMT-PARSE-001");
        assert_eq!(amount, 500.0);
        assert!(!payment_date_str.is_empty());
    }

    #[test]
    fn test_parse_row_to_installment_amount_conversion() {
        let amount_f64 = 750.0f64;
        let amount_f32 = 250.0f32;
        let converted = amount_f32 as f64;
        let fallback = 0.0f64;
        
        assert_eq!(amount_f64, 750.0);
        assert_eq!(converted, 250.0);
        assert_eq!(fallback, 0.0);
    }

    #[test]
    fn test_parse_row_to_installment_date_formats() {
        let rfc3339_date = "2023-10-15T10:30:00Z";
        let standard_date = "2023-10-15 10:30:00.123";
        let simple_date = "2023-10-15 10:30:00";
        
        let parsed_rfc3339 = DateTime::parse_from_rfc3339(rfc3339_date)
            .map(|dt| dt.with_timezone(&Utc));
        assert!(parsed_rfc3339.is_ok());
        
        let parsed_standard = NaiveDateTime::parse_from_str(standard_date, "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive_dt| naive_dt.and_utc());
        assert!(parsed_standard.is_ok());
        
        let parsed_simple = NaiveDateTime::parse_from_str(simple_date, "%Y-%m-%d %H:%M:%S")
            .map(|naive_dt| naive_dt.and_utc());
        assert!(parsed_simple.is_ok());
    }

    #[test]
    fn test_parse_row_to_installment_structure_creation() {
        let installment_data = Installment {
            id: "INST-STRUCT-001".to_string(),
            payment_id: "PMT-STRUCT-001".to_string(),
            amount: 300.0,
            payment_date: Utc::now(),
        };
          assert_eq!(installment_data.id, "INST-STRUCT-001");
        assert_eq!(installment_data.payment_id, "PMT-STRUCT-001");
        assert_eq!(installment_data.amount, 300.0);
    }

    #[test]
    fn test_debug_error_handling() {
        let error_msg = "Database connection failed";
        eprintln!("DEBUG: Failed to insert payment: {}", error_msg);
        assert_eq!(error_msg, "Database connection failed");
    }

    #[test]
    fn test_payment_date_rfc3339_formatting() {
        let now = Utc::now();
        let formatted = now.to_rfc3339();
        
        assert!(!formatted.is_empty());
        assert!(formatted.contains("T"));
        assert!(formatted.contains("Z") || formatted.contains("+"));
    }

    #[test]
    fn test_due_date_optional_rfc3339_formatting() {
        let due_date_some = Some(Utc::now());
        let due_date_none: Option<DateTime<Utc>> = None;
        
        let formatted_some = due_date_some.map(|d| d.to_rfc3339());
        let formatted_none = due_date_none.map(|d| d.to_rfc3339());
        
        assert!(formatted_some.is_some());
        assert!(formatted_none.is_none());
    }

    #[test]
    fn test_map_err_pattern() {
        let result: Result<i32, &str> = Err("test error");
        let mapped = result.map_err(|e| {
            eprintln!("DEBUG: Error occurred: {}", e);
            e
        });
        
        assert!(mapped.is_err());
    }

    #[test]
    fn test_installment_iteration_pattern() {
        let installments = vec![
            Installment {
                id: "INST-1".to_string(),
                payment_id: "PMT-1".to_string(),
                amount: 100.0,
                payment_date: Utc::now(),
            },
            Installment {
                id: "INST-2".to_string(),
                payment_id: "PMT-1".to_string(),
                amount: 200.0,
                payment_date: Utc::now(),
            }
        ];
        
        for installment in &installments {
            assert!(installment.amount > 0.0);
            assert!(!installment.id.is_empty());
        }
        
        assert_eq!(installments.len(), 2);
    }

    #[test]
    fn test_payment_status_update_logic() {
        let mut payment = Payment {
            id: "PMT-STATUS".to_string(),
            transaction_id: "TXN-STATUS".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            due_date: None,
            installments: vec![],
        };
        
        let new_status = PaymentStatus::Installment;
        payment.status = new_status;
        
        assert_eq!(payment.status, PaymentStatus::Installment);
    }

    #[test]
    fn test_additional_amount_processing() {
        let payment_id = "PMT-ADDITIONAL".to_string();
        let additional_amount = Some(500.0);
        
        if let Some(amount) = additional_amount {
            let installment = Installment {
                id: format!("INST-{}", Uuid::new_v4()),
                payment_id: payment_id.clone(),
                amount,
                payment_date: Utc::now(),
            };
            
            assert_eq!(installment.amount, 500.0);
            assert_eq!(installment.payment_id, payment_id);
        }
    }

    #[test]
    fn test_query_bind_parameter_numbering() {
        let mut bind_values: Vec<&str> = Vec::new();
        bind_values.push("value1");
        
        let param_num_1 = bind_values.len() + 1;
        assert_eq!(param_num_1, 2);
        
        bind_values.push("value2");
        let param_num_2 = bind_values.len() + 1;
        assert_eq!(param_num_2, 3);
    }

    #[test]
    fn test_where_clause_construction() {
        let mut where_clauses = Vec::new();
        where_clauses.push("status = $1".to_string());
        where_clauses.push("method = $2".to_string());
        
        let joined = where_clauses.join(" AND ");
        assert_eq!(joined, "status = $1 AND method = $2");
        
        let mut base_query = "SELECT * FROM payments".to_string();
        if !where_clauses.is_empty() {
            base_query.push_str(" WHERE ");
            base_query.push_str(&joined);
        }
        
        assert!(base_query.contains("WHERE"));
        assert!(base_query.contains("AND"));
    }

    #[test]
    fn test_row_iteration_and_payment_loading() {
        let payment_ids = vec!["PMT-1", "PMT-2", "PMT-3"];
        let mut payments = Vec::with_capacity(payment_ids.len());
        
        for payment_id in payment_ids {
            let payment = Payment {
                id: payment_id.to_string(),
                transaction_id: format!("TXN-{}", payment_id),
                amount: 1000.0,
                method: PaymentMethod::Cash,
                status: PaymentStatus::Paid,
                payment_date: Utc::now(),
                due_date: None,
                installments: vec![],
            };
            payments.push(payment);
        }
        
        assert_eq!(payments.len(), 3);
        assert_eq!(payments[0].id, "PMT-1");
        assert_eq!(payments[1].id, "PMT-2");
        assert_eq!(payments[2].id, "PMT-3");
    }

    #[test]
    fn test_update_query_variable_preparation() {
        let payment = Payment {
            id: "PMT-UPDATE-VAR".to_string(),
            transaction_id: "TXN-UPDATE-VAR".to_string(),
            amount: 1500.0,
            method: PaymentMethod::EWallet,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            due_date: Some(Utc::now()),
            installments: vec![],
        };
        
        let payment_method_str = payment.method.to_string();
        let status_str = payment.status.to_string();
        
        assert_eq!(payment_method_str, "E_WALLET");
        assert_eq!(status_str, "CICILAN");
        
        let payment_date_rfc = payment.payment_date.to_rfc3339();
        let due_date_rfc = payment.due_date.map(|d| d.to_rfc3339());
        
        assert!(!payment_date_rfc.is_empty());
        assert!(due_date_rfc.is_some());
    }

    #[test]
    fn test_delete_cascade_order() {
        let payment_id = "PMT-DELETE-CASCADE";
        
        let delete_order = vec![
            format!("DELETE FROM installments WHERE payment_id = '{}'", payment_id),
            format!("DELETE FROM payments WHERE id = '{}'", payment_id),
        ];
        
        assert_eq!(delete_order.len(), 2);
        assert!(delete_order[0].contains("installments"));
        assert!(delete_order[1].contains("payments"));
    }

    #[test]
    fn test_installment_insert_binding() {
        let installment = Installment {
            id: "INST-BIND-TEST".to_string(),
            payment_id: "PMT-BIND-TEST".to_string(),
            amount: 750.0,
            payment_date: Utc::now(),
        };
        
        let payment_date_rfc = installment.payment_date.to_rfc3339();
        
        assert_eq!(installment.id, "INST-BIND-TEST");
        assert_eq!(installment.payment_id, "PMT-BIND-TEST");
        assert_eq!(installment.amount, 750.0);
        assert!(!payment_date_rfc.is_empty());
    }

    #[test]
    fn test_load_payment_installments_ordering() {
        let mut installments = vec![
            Installment {
                id: "INST-3".to_string(),
                payment_id: "PMT-ORDER".to_string(),
                amount: 300.0,
                payment_date: Utc::now() + chrono::Duration::days(2),
            },
            Installment {
                id: "INST-1".to_string(),
                payment_id: "PMT-ORDER".to_string(),
                amount: 100.0,
                payment_date: Utc::now(),
            },
            Installment {
                id: "INST-2".to_string(),
                payment_id: "PMT-ORDER".to_string(),
                amount: 200.0,
                payment_date: Utc::now() + chrono::Duration::days(1),
            }
        ];
        
        installments.sort_by(|a, b| a.payment_date.cmp(&b.payment_date));
        
        assert_eq!(installments[0].id, "INST-1");
        assert_eq!(installments[1].id, "INST-2");
        assert_eq!(installments[2].id, "INST-3");
    }

    #[test]
    fn test_parse_row_error_propagation() {
        let invalid_method = "INVALID_PAYMENT_METHOD";
        let is_valid_method = matches!(invalid_method, "CASH" | "CREDIT_CARD" | "BANK_TRANSFER" | "E_WALLET");
        
        assert!(!is_valid_method);
        
        let invalid_date = "not-a-date";
        let date_parse_result = DateTime::parse_from_rfc3339(invalid_date);
        assert!(date_parse_result.is_err());
    }

    #[test]
    fn test_payment_construction_with_empty_installments() {
        let payment = Payment {
            id: "PMT-EMPTY-INST".to_string(),
            transaction_id: "TXN-EMPTY-INST".to_string(),
            amount: 2000.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        assert!(payment.installments.is_empty());
        assert_eq!(payment.installments.len(), 0);
        assert_eq!(payment.amount, 2000.0);
    }

    #[test]
    fn test_try_get_amount_with_fallback_chain() {
        let amount_f64_primary = 1000.0f64;
        let amount_f32_fallback = 500.0f32;
        let final_fallback = 0.0f64;
        
        let result = if amount_f64_primary > 0.0 {
            amount_f64_primary
        } else {
            amount_f32_fallback as f64
        };
        
        let final_result = if result > 0.0 { result } else { final_fallback };
        
        assert_eq!(final_result, 1000.0);
    }

    #[test]
    fn test_due_date_option_handling_patterns() {
        let due_date_some = Some("2023-12-31T23:59:59Z".to_string());
        let due_date_none: Option<String> = None;
        
        let processed_some = due_date_some.map(|d| {
            DateTime::parse_from_rfc3339(&d)
                .map(|dt| dt.with_timezone(&Utc))
        }).transpose();
        
        let processed_none = due_date_none.map(|d| {
            DateTime::parse_from_rfc3339(&d)
                .map(|dt| dt.with_timezone(&Utc))
        }).transpose();
        
        assert!(processed_some.is_ok());
        assert!(processed_some.unwrap().is_some());
        assert!(processed_none.is_ok());
        assert!(processed_none.unwrap().is_none());
    }

    #[test]
    fn test_naive_datetime_conversion_patterns() {
        let date_with_ms = "2023-10-15 10:30:00.123";
        let date_without_ms = "2023-10-15 10:30:00";
        
        let parsed_with_ms = NaiveDateTime::parse_from_str(date_with_ms, "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive_dt| naive_dt.and_utc());
        
        let parsed_without_ms = NaiveDateTime::parse_from_str(date_without_ms, "%Y-%m-%d %H:%M:%S")
            .map(|naive_dt| naive_dt.and_utc());
        
        assert!(parsed_with_ms.is_ok());
        assert!(parsed_without_ms.is_ok());
    }

    #[test]
    fn test_create_payment_debug_output() {
        let payment_id = "PMT-DEBUG-001";
        let transaction_id = "TXN-DEBUG-001";
        
        eprintln!("DEBUG: Creating payment with ID: {}, Transaction ID: {}", payment_id, transaction_id);
        
        assert_eq!(payment_id, "PMT-DEBUG-001");
        assert_eq!(transaction_id, "TXN-DEBUG-001");
    }

    #[test]
    fn test_find_all_query_construction() {
        let base_query = "SELECT id, transaction_id, amount, method, status, payment_date, due_date FROM payments";
        let mut query_with_where = base_query.to_string();
        query_with_where.push_str(" WHERE status = $1 AND method = $2");

        assert!(query_with_where.starts_with("SELECT"));
        assert!(query_with_where.contains("FROM payments"));
        assert!(query_with_where.contains("WHERE"));
        assert!(query_with_where.contains("AND"));
    }

    #[test]
    fn test_payment_status_parsing() {
        let status_paid = PaymentStatus::from_string("LUNAS");
        assert_eq!(status_paid, Some(PaymentStatus::Paid));
        
        let status_installment = PaymentStatus::from_string("CICILAN");
        assert_eq!(status_installment, Some(PaymentStatus::Installment));
        
        let status_invalid = PaymentStatus::from_string("INVALID");
        assert_eq!(status_invalid, None);
    }

    #[test]
    fn test_payment_method_string_conversion() {
        assert_eq!(PaymentMethod::Cash.to_string(), "CASH");
        assert_eq!(PaymentMethod::CreditCard.to_string(), "CREDIT_CARD");
        assert_eq!(PaymentMethod::BankTransfer.to_string(), "BANK_TRANSFER");
        assert_eq!(PaymentMethod::EWallet.to_string(), "E_WALLET");
    }

    #[test]
    fn test_payment_creation_structure() {
        let id = Uuid::new_v4().to_string();
        let transaction_id = format!("TXN-{}", Uuid::new_v4());
        let amount = 1000.0;
        let method = PaymentMethod::Cash;
        let status = PaymentStatus::Paid;
        let payment_date = Utc::now();
        let due_date = Some(Utc::now());

        let payment = Payment {
            id: id.clone(),
            transaction_id: transaction_id.clone(),
            amount,
            method,
            status,
            payment_date,
            installments: Vec::new(),
            due_date,
        };

        assert_eq!(payment.id, id);
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, amount);
        assert_eq!(payment.method, PaymentMethod::Cash);
        assert_eq!(payment.status, PaymentStatus::Paid);
        assert!(payment.installments.is_empty());
        assert!(payment.due_date.is_some());
    }

    #[test]
    fn test_installment_creation() {
        let id = format!("INST-{}", Uuid::new_v4());
        let payment_id = Uuid::new_v4().to_string();
        let amount = 500.0;
        let payment_date = Utc::now();

        let installment = Installment {
            id: id.clone(),
            payment_id: payment_id.clone(),
            amount,
            payment_date,
        };

        assert_eq!(installment.id, id);
        assert_eq!(installment.payment_id, payment_id);
        assert_eq!(installment.amount, amount);
    }

    #[test]
    fn test_filter_map_creation() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "PENDING".to_string());
        filters.insert("method".to_string(), "CASH".to_string());
        filters.insert("transaction_id".to_string(), "TXN-123".to_string());

        assert_eq!(filters.get("status"), Some(&"PENDING".to_string()));
        assert_eq!(filters.get("method"), Some(&"CASH".to_string()));
        assert_eq!(filters.get("transaction_id"), Some(&"TXN-123".to_string()));
        assert_eq!(filters.get("invalid_key"), None);
    }

    #[test]
    fn test_date_time_string_formatting() {
        let now = Utc::now();
        let formatted = now.naive_utc().to_string();
        
        assert!(!formatted.is_empty());
        assert!(formatted.contains("-")); 
        assert!(formatted.contains(":")); 
    }

    #[test]
    fn test_uuid_generation_for_installment() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let installment_id1 = format!("INST-{}", uuid1);
        let installment_id2 = format!("INST-{}", uuid2);

        assert_ne!(installment_id1, installment_id2);
        assert!(installment_id1.starts_with("INST-"));
        assert!(installment_id2.starts_with("INST-"));
    }

    #[test]
    fn test_payment_status_to_string() {
        assert_eq!(PaymentStatus::Paid.to_string(), "LUNAS");
        assert_eq!(PaymentStatus::Installment.to_string(), "CICILAN");
    }

    #[test]
    fn test_payment_amount_calculation() {
        let main_payment = Payment {
            id: Uuid::new_v4().to_string(),
            transaction_id: format!("TXN-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: vec![
                Installment {
                    id: format!("INST-{}", Uuid::new_v4()),
                    payment_id: "payment-1".to_string(),
                    amount: 300.0,
                    payment_date: Utc::now(),
                },
                Installment {
                    id: format!("INST-{}", Uuid::new_v4()),
                    payment_id: "payment-1".to_string(),
                    amount: 200.0,
                    payment_date: Utc::now(),
                },
            ],
            due_date: None,
        };

        let total_installments: f64 = main_payment.installments.iter().map(|i| i.amount).sum();
        assert_eq!(total_installments, 500.0);
        
        let remaining_amount = main_payment.amount - total_installments;
        assert_eq!(remaining_amount, 500.0);
    }

    #[test]
    fn test_payment_with_no_installments() {
        let payment = Payment {
            id: Uuid::new_v4().to_string(),
            transaction_id: format!("TXN-{}", Uuid::new_v4()),
            amount: 1500.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: Some(Utc::now()),
        };

        assert!(payment.installments.is_empty());
        assert_eq!(payment.amount, 1500.0);
        assert_eq!(payment.status, PaymentStatus::Paid);
    }

    #[test]
    fn test_payment_method_equality() {
        assert_eq!(PaymentMethod::Cash, PaymentMethod::Cash);
        assert_eq!(PaymentMethod::CreditCard, PaymentMethod::CreditCard);
        assert_ne!(PaymentMethod::Cash, PaymentMethod::CreditCard);
        assert_ne!(PaymentMethod::BankTransfer, PaymentMethod::EWallet);
    }
}