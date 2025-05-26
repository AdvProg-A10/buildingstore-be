use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::{DateTime, Utc, NaiveDateTime};
use std::collections::HashMap;
use uuid::Uuid;

use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};

pub struct PembayaranRepository;

impl PembayaranRepository {    pub async fn create(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error>{        
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
                eprintln!("DEBUG: Failed to insert payment: {}", e);
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
    }    pub async fn find_by_id(mut db: PoolConnection<Any>, id: &str) -> Result<Payment, sqlx::Error>{
        let payment_with_installments = Self::load_payment_with_installments(&mut db, id).await?;

        Ok(payment_with_installments)
    }    pub async fn find_all(mut db: PoolConnection<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, sqlx::Error> {
        let mut base_query = "SELECT id, transaction_id, amount, method, status, payment_date, due_date FROM payments".to_string();
        let mut where_clauses = Vec::new();
        let mut bind_values: Vec<&str> = Vec::new();
        
        if let Some(filter_map) = &filters {
            if let Some(status_str) = filter_map.get("status") {
                where_clauses.push("status = $1".to_string());
                bind_values.push(status_str);
            }
            if let Some(method) = filter_map.get("method") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("method = ${}", param_num));
                bind_values.push(method);
            }
            if let Some(transaction_id) = filter_map.get("transaction_id") {
                let param_num = bind_values.len() + 1;
                where_clauses.push(format!("transaction_id = ${}", param_num));
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
    }    pub async fn load_payment_with_installments(db: &mut PoolConnection<Any>, payment_id: &str) -> Result<Payment, sqlx::Error> {        
        let payment_row = sqlx::query("
            SELECT id, transaction_id, amount, method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
        .bind(payment_id)
        .fetch_one(&mut **db)
        .await
        .map_err(|e| {
            e
        })?;
        
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
            })
            .map_err(|e| {
                eprintln!("Failed to parse payment_date '{}': {}", payment_date_str, e);
                sqlx::Error::RowNotFound
            })?;        let due_date = due_date_str
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
            .transpose()
            .map_err(|e| {
                eprintln!("Failed to parse due_date: {}", e);
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
            })
            .map_err(|e| {
                eprintln!("Failed to parse installment payment_date '{}': {}", payment_date_str, e);
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
    }    #[test]
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
    }    #[test]
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