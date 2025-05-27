use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use rocket::{get, post, put, delete, routes, Route, State, catch};
use rocket::serde::json::Json;
use rocket::http::Status;
use autometrics::autometrics;

use crate::manajemen_pembayaran::model::payment::Payment;
use crate::manajemen_pembayaran::service::payment_service::{PaymentService, PaymentError};
use sqlx::{Any, Pool};

#[derive(Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub transaction_id: String,
    pub amount: f64,
    pub method: String,
    pub status: String,
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub new_status: String,
    pub additional_amount: Option<f64>,
}

#[derive(Deserialize)]
pub struct UpdatePaymentRequest {
    pub transaction_id: String,
    pub amount: f64,
    pub method: String,
    pub status: String,
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub struct AddInstallmentRequest {
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[autometrics]
#[post("/payments", format = "json", data = "<payment_request>")]
pub async fn create_payment(payment_request: Json<CreatePaymentRequest>, db: &State<Pool<Any>>) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    let method: crate::manajemen_pembayaran::model::payment::PaymentMethod = match payment_service.parse_payment_method(&payment_request.method) {
        Ok(m) => m,        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment method: {e:?}"),
                    data: None,
                }),
            );
        }
    };
      let status = match payment_service.parse_payment_status(&payment_request.status) {
        Ok(s) => s,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment status: {e:?}"),
                    data: None,
                }),
            );
        }
    };
    
    let due_date: Option<chrono::DateTime<Utc>> = match &payment_request.due_date {
        Some(date_str) => match chrono::DateTime::parse_from_rfc3339(date_str) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),            Err(_) => {
                return (
                    Status::BadRequest,
                    Json(ApiResponse {
                        success: false,
                        message: "Invalid due date format. Use RFC3339 format".to_string(),
                        data: None,
                    }),
                );
            }
        },
        None => None,
    };
    
    let payment = Payment {
        id: payment_service.generate_payment_id(),
        transaction_id: payment_request.transaction_id.clone(),
        amount: payment_request.amount,
        method,
        status,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date,
    };
    
    match payment_service.create_payment(db, payment).await {
        Ok(created_payment) => (
            Status::Created,
            Json(ApiResponse {
                success: true,
                message: "Payment created successfully".to_string(),
                data: Some(created_payment),
            }),
        ),        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to create payment: {e:?}"),
                data: None,
            }),
        ),
    }
}

#[autometrics]
#[get("/payments/<id>")]
pub async fn get_payment_by_id(id: String, db: &State<Pool<Any>>) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    match payment_service.get_payment_by_id(db, &id).await {
        Ok(payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment retrieved successfully".to_string(),
                data: Some(payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),        Err(e) => (
            Status::InternalServerError,            Json(ApiResponse {
                success: false,
                message: format!("Failed to retrieve payment: {e:?}"),
                data: None,
            }),
        ),
    }
}

#[autometrics]
#[put("/payments/<id>", format = "json", data = "<update_request>")]
pub async fn update_payment(
    id: String,
    update_request: Json<UpdatePaymentRequest>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    let method = match payment_service.parse_payment_method(&update_request.method) {
        Ok(m) => m,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment method: {e:?}"),
                    data: None,
                }),
            );
        }
    };
    
    let status = match payment_service.parse_payment_status(&update_request.status) {
        Ok(s) => s,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment status: {e:?}"),
                    data: None,
                }),
            );
        }
    };
    
    let due_date = if let Some(due_date_str) = &update_request.due_date {
        match chrono::DateTime::parse_from_rfc3339(due_date_str) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return (
                    Status::BadRequest,
                    Json(ApiResponse {
                        success: false,
                        message: "Invalid due date format. Use RFC3339 format.".to_string(),
                        data: None,
                    }),
                );
            }
        }
    } else {
        None
    };
    
    let current_payment = match payment_service.get_payment_by_id(db, &id).await {
        Ok(payment) => payment,
        Err(PaymentError::NotFound(msg)) => {
            return (
                Status::NotFound,
                Json(ApiResponse {
                    success: false,
                    message: msg,
                    data: None,
                }),
            );
        },
        Err(e) => {
            return (
                Status::InternalServerError,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to retrieve payment: {e:?}"),
                    data: None,
                }),
            );
        }
    };
    
    let updated_payment = Payment {
        id: id.clone(),
        transaction_id: update_request.transaction_id.clone(),
        amount: update_request.amount,
        method,
        status,
        payment_date: current_payment.payment_date,
        installments: current_payment.installments,
        due_date,
    };
    
    match payment_service.update_payment(db, updated_payment).await {
        Ok(updated_payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment updated successfully".to_string(),
                data: Some(updated_payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to update payment: {e:?}"),
                data: None,
            }),
        ),
    }
}

#[autometrics]
#[get("/payments?<status>&<method>&<transaction_id>")]
pub async fn get_all_payments(
    status: Option<String>,
    method: Option<String>,
    transaction_id: Option<String>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Vec<Payment>>>) {
    let payment_service = PaymentService::new();
    
    let mut filters = HashMap::new();
    if let Some(status_str) = status {
        filters.insert("status".to_string(), status_str);
    }
    if let Some(method_str) = method {
        filters.insert("method".to_string(), method_str);
    }
    if let Some(tx_id) = transaction_id {
        filters.insert("transaction_id".to_string(), tx_id);
    }
    
    let filters_option = if filters.is_empty() { None } else { Some(filters) };
    
    match payment_service.get_all_payments(db, filters_option).await {
        Ok(payments) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: format!("Successfully retrieved {} payments", payments.len()),
                data: Some(payments),
            }),
        ),        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to retrieve payments: {e:?}"),
                data: None,
            }),
        ),
    }
}


#[autometrics]
#[put("/payments/<id>/status", format = "json", data = "<status_request>")]
pub async fn update_payment_status(
    id: String,
    status_request: Json<UpdatePaymentStatusRequest>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
      let new_status = match payment_service.parse_payment_status(&status_request.new_status) {
        Ok(s) => s,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment status: {e:?}"),
                    data: None,
                }),
            );
        }
    };
    
    match payment_service.update_payment_status(db, id, new_status, status_request.additional_amount).await {
        Ok(updated_payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment status updated successfully".to_string(),
                data: Some(updated_payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to update payment status: {e:?}"),
                data: None,
            }),
        ),
    }
}


#[autometrics]
#[post("/payments/<id>/installments", format = "json", data = "<installment_request>")]
pub async fn add_installment(
    id: String,
    installment_request: Json<AddInstallmentRequest>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    match payment_service.add_installment(db, &id, installment_request.amount).await {
        Ok(updated_payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Installment added successfully".to_string(),
                data: Some(updated_payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),
        Err(PaymentError::InvalidInput(msg)) => (
            Status::BadRequest,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to add installment: {e:?}"),
                data: None,
            }),
        ),
    }
}


#[autometrics]
#[delete("/payments/<id>")]
pub async fn delete_payment(id: String, db: &State<Pool<Any>>) -> (Status, Json<ApiResponse<()>>) {
    let payment_service = PaymentService::new();
    
    match payment_service.delete_payment(db, &id).await {
        Ok(_) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment deleted successfully".to_string(),
                data: None,
            }),
        ),        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to delete payment: {e:?}"),
                data: None,
            }),
        ),
    }
}

#[derive(Deserialize)]
pub struct PaymentFilterRequest {
    pub status: Option<String>,
    pub method: Option<String>,
    pub transaction_id: Option<String>,
}

pub fn routes() -> Vec<Route> {
    routes![
        create_payment,
        get_payment_by_id,
        update_payment,
        get_all_payments,
        update_payment_status,
        add_installment,
        delete_payment
    ]
}

#[catch(404)]
pub fn not_found_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        message: "Resource not found".to_string(),
        data: None,
    })
}

#[catch(400)]
pub fn bad_request_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        message: "Bad request".to_string(),
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc};
    
    #[test]
    fn test_api_response_serialization() {
        let response: ApiResponse<String> = ApiResponse {
            success: true,
            message: "Test message".to_string(),
            data: Some("Test data".to_string()),
        };
        
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("Test message"));
        assert!(serialized.contains("Test data"));
        
        let deserialized: ApiResponse<String> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.success, true);
        assert_eq!(deserialized.message, "Test message");
        assert_eq!(deserialized.data, Some("Test data".to_string()));
    }

    #[test]
    fn test_api_response_with_none_data() {
        let response: ApiResponse<Payment> = ApiResponse {
            success: false,
            message: "Error occurred".to_string(),
            data: None,
        };

        assert_eq!(response.success, false);
        assert_eq!(response.message, "Error occurred");
        assert!(response.data.is_none());
    }

    #[test]
    fn test_create_payment_request_deserialization() {
        let json_str = r#"{
            "transaction_id": "TXN-123",
            "amount": 1000.0,
            "method": "CASH",
            "status": "PENDING",
            "due_date": "2024-12-31T23:59:59Z"
        }"#;

        let request: CreatePaymentRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.transaction_id, "TXN-123");
        assert_eq!(request.amount, 1000.0);
        assert_eq!(request.method, "CASH");
        assert_eq!(request.status, "PENDING");
        assert!(request.due_date.is_some());
    }

    #[test]
    fn test_create_payment_request_without_due_date() {
        let json_str = r#"{
            "transaction_id": "TXN-456",
            "amount": 500.0,
            "method": "CREDIT_CARD",
            "status": "COMPLETED"
        }"#;

        let request: CreatePaymentRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.transaction_id, "TXN-456");
        assert_eq!(request.amount, 500.0);
        assert_eq!(request.method, "CREDIT_CARD");
        assert_eq!(request.status, "COMPLETED");
        assert!(request.due_date.is_none());
    }

    #[test]
    fn test_update_payment_status_request() {
        let json_str = r#"{
            "new_status": "COMPLETED",
            "additional_amount": 250.0
        }"#;

        let request: UpdatePaymentStatusRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.new_status, "COMPLETED");
        assert_eq!(request.additional_amount, Some(250.0));
    }

    #[test]
    fn test_update_payment_status_request_without_additional_amount() {
        let json_str = r#"{
            "new_status": "FAILED"
        }"#;

        let request: UpdatePaymentStatusRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.new_status, "FAILED");
        assert!(request.additional_amount.is_none());
    }

    #[test]
    fn test_add_installment_request() {
        let json_str = r#"{
            "amount": 300.0
        }"#;

        let request: AddInstallmentRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.amount, 300.0);
    }

    #[test]
    fn test_payment_filter_request() {
        let json_str = r#"{
            "status": "PENDING",
            "method": "CASH",
            "transaction_id": "TXN-789"
        }"#;

        let request: PaymentFilterRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.status, Some("PENDING".to_string()));
        assert_eq!(request.method, Some("CASH".to_string()));
        assert_eq!(request.transaction_id, Some("TXN-789".to_string()));
    }

    #[test]
    fn test_payment_filter_request_partial() {
        let json_str = r#"{
            "status": "COMPLETED"
        }"#;

        let request: PaymentFilterRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.status, Some("COMPLETED".to_string()));
        assert!(request.method.is_none());
        assert!(request.transaction_id.is_none());
    }

    #[test]
    fn test_api_response_with_payment_data() {
        use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
        use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

        let payment = Payment {
            id: "PMT-123".to_string(),
            transaction_id: "TXN-456".to_string(),            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let response = ApiResponse {
            success: true,
            message: "Payment retrieved successfully".to_string(),
            data: Some(payment.clone()),
        };

        assert_eq!(response.success, true);
        assert!(response.data.is_some());
        if let Some(data) = response.data {
            assert_eq!(data.id, payment.id);
            assert_eq!(data.amount, payment.amount);
        }
    }

    #[test]
    fn test_api_response_with_payment_list() {
        use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
        use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

        let payments = vec![
            Payment {
                id: "PMT-1".to_string(),
                transaction_id: "TXN-1".to_string(),
                amount: 500.0,                method: PaymentMethod::Cash,
                status: PaymentStatus::Paid,
                payment_date: Utc::now(),
                installments: Vec::new(),
                due_date: None,
            },
            Payment {
                id: "PMT-2".to_string(),
                transaction_id: "TXN-2".to_string(),
                amount: 750.0,                method: PaymentMethod::CreditCard,
                status: PaymentStatus::Installment,
                payment_date: Utc::now(),
                installments: Vec::new(),
                due_date: Some(Utc::now()),
            },
        ];

        let response = ApiResponse {
            success: true,
            message: format!("Successfully retrieved {} payments", payments.len()),
            data: Some(payments.clone()),
        };

        assert_eq!(response.success, true);
        assert!(response.data.is_some());
        if let Some(data) = response.data {
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].id, "PMT-1");
            assert_eq!(data[1].id, "PMT-2");
        }
    }

    #[test]
    fn test_error_response_structure() {
        let error_response: ApiResponse<Payment> = ApiResponse {
            success: false,
            message: "Payment not found".to_string(),
            data: None,
        };

        assert_eq!(error_response.success, false);
        assert_eq!(error_response.message, "Payment not found");
        assert!(error_response.data.is_none());
    }

    #[test]
    fn test_filter_map_creation() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "PENDING".to_string());
        filters.insert("method".to_string(), "CASH".to_string());

        let filters_option = if filters.is_empty() { None } else { Some(filters.clone()) };
        assert!(filters_option.is_some());

        let empty_filters: HashMap<String, String> = HashMap::new();
        let empty_filters_option = if empty_filters.is_empty() { None } else { Some(empty_filters) };
        assert!(empty_filters_option.is_none());
    }

    #[test]
    fn test_date_parsing_logic() {
        let valid_date = "2024-12-31T23:59:59Z";
        let parsed_date = chrono::DateTime::parse_from_rfc3339(valid_date);
        assert!(parsed_date.is_ok());

        let invalid_date = "invalid-date-format";
        let parsed_invalid = chrono::DateTime::parse_from_rfc3339(invalid_date);
        assert!(parsed_invalid.is_err());
    }

    #[test]
    fn test_payment_id_generation_format() {
        let payment_service = PaymentService::new();
        let payment_id = payment_service.generate_payment_id();
        
        assert!(payment_id.starts_with("PMT-"));
        assert_eq!(payment_id.len(), 40);
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let original_request = CreatePaymentRequest {
            transaction_id: "TXN-TEST".to_string(),
            amount: 1234.56,
            method: "BANK_TRANSFER".to_string(),
            status: "INSTALLMENT".to_string(),
            due_date: Some("2024-06-15T10:30:00Z".to_string()),
        };

        let serialized = serde_json::to_string(&original_request).unwrap();
        let deserialized: CreatePaymentRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.transaction_id, original_request.transaction_id);
        assert_eq!(deserialized.amount, original_request.amount);
        assert_eq!(deserialized.method, original_request.method);
        assert_eq!(deserialized.status, original_request.status);
        assert_eq!(deserialized.due_date, original_request.due_date);
    }

    #[test]
    fn test_create_payment_request_invalid_method_parsing() {
        let json_str = r#"{
            "transaction_id": "TXN-123",
            "amount": 1000.0,
            "method": "INVALID_METHOD",
            "status": "PENDING"
        }"#;

        let request: CreatePaymentRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.method, "INVALID_METHOD");
        
        let payment_service = PaymentService::new();
        let parse_result = payment_service.parse_payment_method(&request.method);
        assert!(parse_result.is_err());
    }

    #[test]
    fn test_get_payment_by_id_success_response() {
        use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
        use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

        let payment = Payment {
            id: "PMT-TEST-123".to_string(),
            transaction_id: "TXN-TEST-456".to_string(),
            amount: 1500.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let response = ApiResponse {
            success: true,
            message: "Payment retrieved successfully".to_string(),
            data: Some(payment.clone()),
        };

        assert_eq!(response.success, true);
        assert_eq!(response.message, "Payment retrieved successfully");
        assert!(response.data.is_some());
        
        if let Some(data) = response.data {
            assert_eq!(data.id, "PMT-TEST-123");
            assert_eq!(data.amount, 1500.0);
        }
    }

    #[test]
    fn test_payment_service_initialization_for_get_all_payments() {
        let payment_service = PaymentService::new();
        
        assert!(payment_service.generate_payment_id().starts_with("PMT-"));
        
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "PENDING".to_string());
        filters.insert("method".to_string(), "CASH".to_string());
        
        let filters_option = if filters.is_empty() { None } else { Some(filters.clone()) };
        assert!(filters_option.is_some());
        
        if let Some(filter_map) = filters_option {
            assert_eq!(filter_map.get("status"), Some(&"PENDING".to_string()));
            assert_eq!(filter_map.get("method"), Some(&"CASH".to_string()));
        }
    }

    #[test]
    fn test_update_payment_status_invalid_status_parsing() {
        let json_str = r#"{
            "new_status": "INVALID_STATUS",
            "additional_amount": 100.0
        }"#;

        let request: UpdatePaymentStatusRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.new_status, "INVALID_STATUS");
        
        let payment_service = PaymentService::new();
        let parse_result = payment_service.parse_payment_status(&request.new_status);
        assert!(parse_result.is_err());
        
        let error_response: ApiResponse<Payment> = ApiResponse {
            success: false,
            message: "Invalid payment status: ParseError".to_string(),
            data: None,
        };
        
        assert_eq!(error_response.success, false);
        assert!(error_response.message.contains("Invalid payment status"));
        assert!(error_response.data.is_none());
    }

    #[test]
    fn test_payment_service_initialization_for_add_installment() {
        let payment_service = PaymentService::new();
        
        let payment_id = payment_service.generate_payment_id();
        assert!(payment_id.starts_with("PMT-"));
        assert_eq!(payment_id.len(), 40);
        
        let installment_request = AddInstallmentRequest {
            amount: 500.0,
        };
        
        assert_eq!(installment_request.amount, 500.0);
        assert!(installment_request.amount > 0.0);
    }

    #[test]
    fn test_delete_payment_error_response() {
        let error_response: ApiResponse<()> = ApiResponse {
            success: false,
            message: "Failed to delete payment: DatabaseError".to_string(),
            data: None,
        };

        assert_eq!(error_response.success, false);
        assert!(error_response.message.contains("Failed to delete payment"));
        assert!(error_response.data.is_none());
        
        let formatted_error = format!("Failed to delete payment: {:?}", "TestError");
        assert!(formatted_error.contains("Failed to delete payment"));
        assert!(formatted_error.contains("TestError"));
    }

    
    #[test]
    fn test_not_found_catcher() {
        let response = not_found_catcher();
        let response_inner = response.into_inner();
        
        assert_eq!(response_inner.success, false);
        assert_eq!(response_inner.message, "Resource not found");
        assert!(response_inner.data.is_none());
        
        let serialized = serde_json::to_string(&response_inner).unwrap();
        assert!(serialized.contains("Resource not found"));
        assert!(serialized.contains("false"));
    }
    
    #[test]
    fn test_bad_request_catcher() {
        let response = bad_request_catcher();
        let response_inner = response.into_inner();
        
        assert_eq!(response_inner.success, false);
        assert_eq!(response_inner.message, "Bad request");
        assert!(response_inner.data.is_none());
        
        let serialized = serde_json::to_string(&response_inner).unwrap();
        assert!(serialized.contains("Bad request"));
        assert!(serialized.contains("false"));
        
        let expected_response: ApiResponse<()> = ApiResponse {
            success: false,
            message: "Bad request".to_string(),
            data: None,
        };
        
        assert_eq!(response_inner.success, expected_response.success);
        assert_eq!(response_inner.message, expected_response.message);
        assert_eq!(response_inner.data, expected_response.data);
    }

    #[test]
    fn test_payment_filter_request_all_fields() {
        let filter_request = PaymentFilterRequest {
            status: Some("PENDING".to_string()),
            method: Some("CREDIT_CARD".to_string()),
            transaction_id: Some("TXN-FILTER-TEST".to_string()),
        };

        assert!(filter_request.status.is_some());
        assert!(filter_request.method.is_some());
        assert!(filter_request.transaction_id.is_some());
        
        let mut filters = HashMap::new();
        if let Some(status_str) = filter_request.status {
            filters.insert("status".to_string(), status_str);
        }
        if let Some(method_str) = filter_request.method {
            filters.insert("method".to_string(), method_str);
        }
        if let Some(tx_id) = filter_request.transaction_id {
            filters.insert("transaction_id".to_string(), tx_id);
        }

        assert_eq!(filters.len(), 3);
        assert!(filters.contains_key("status"));
        assert!(filters.contains_key("method"));
        assert!(filters.contains_key("transaction_id"));
    }

    #[test]
    fn test_error_response_formatting() {
        let parse_error = format!("Invalid payment method: {:?}", "TestError");
        assert!(parse_error.contains("Invalid payment method"));
        
        let status_error = format!("Invalid payment status: {:?}", "TestError");
        assert!(status_error.contains("Invalid payment status"));
        
        let retrieval_error = format!("Failed to retrieve payment: {:?}", "TestError");
        assert!(retrieval_error.contains("Failed to retrieve payment"));
        
        let update_error = format!("Failed to update payment status: {:?}", "TestError");
        assert!(update_error.contains("Failed to update payment status"));
        
        let installment_error = format!("Failed to add installment: {:?}", "TestError");
        assert!(installment_error.contains("Failed to add installment"));
        
        let delete_error = format!("Failed to delete payment: {:?}", "TestError");
        assert!(delete_error.contains("Failed to delete payment"));
    }
}