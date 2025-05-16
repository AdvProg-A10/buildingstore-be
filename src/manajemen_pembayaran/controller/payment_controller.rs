use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rocket::{get, post, put, delete, routes, Route, State, catch, catchers};
use rocket::serde::{json::Json};
use rocket::serde::json::serde_json;
use rocket::http::Status;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::service::payment_service::PaymentService;

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub transaction_id: String,
    pub amount: f64,
    pub method: String,
}

#[derive(Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub payment_id: String,
    pub new_status: String,
    pub additional_amount: Option<f64>,
}

#[derive(Deserialize)]
pub struct AddInstallmentRequest {
    pub payment_id: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct PaymentFilterRequest {
    pub status: Option<String>,
    pub method: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

pub struct PaymentController {
    service: Arc<dyn PaymentService>,
}

impl PaymentController {
    pub fn new(service: Arc<dyn PaymentService>) -> Self {
        Self { service }
    }

    pub fn create_payment(&self, request: CreatePaymentRequest) -> ApiResponse<Payment> {
        let method = match request.method.to_uppercase().as_str() {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "BANK_TRANSFER" => PaymentMethod::BankTransfer,
            "E_WALLET" => PaymentMethod::EWallet,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Metode pembayaran tidak valid".to_string()),
                }
            }
        };

        match self.service.create_payment(request.transaction_id, request.amount, method) {
            Ok(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }

    pub fn update_payment_status(&self, request: UpdatePaymentStatusRequest) -> ApiResponse<Payment> {
        let status = match PaymentStatus::from_string(&request.new_status) {
            Some(status) => status,
            None => {
                return ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Status pembayaran tidak valid".to_string()),
                }
            }
        };

        match self.service.update_payment_status(
            request.payment_id,
            status,
            request.additional_amount,
        ) {
            Ok(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }

    pub fn add_installment(&self, request: AddInstallmentRequest) -> ApiResponse<Payment> {
        match self.service.add_installment(&request.payment_id, request.amount) {
            Ok(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }

    pub fn get_payment(&self, payment_id: &str) -> ApiResponse<Payment> {
        match self.service.get_payment(payment_id) {
            Some(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            None => ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Pembayaran dengan ID {} tidak ditemukan", payment_id)),
            },
        }
    }

    pub fn get_payment_by_transaction(&self, transaction_id: &str) -> ApiResponse<Payment> {
        match self.service.get_payment_by_transaction(transaction_id) {
            Some(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            None => ApiResponse {
                success: false,
                data: None,
                message: Some(format!(
                    "Pembayaran untuk transaksi {} tidak ditemukan",
                    transaction_id
                )),
            },
        }
    }

    pub fn get_all_payments(&self, filter: Option<PaymentFilterRequest>) -> ApiResponse<Vec<Payment>> {
        let mut filters: Option<HashMap<String, String>> = None;
        
        if let Some(filter_req) = filter {
            let mut map = HashMap::new();
            
            if let Some(status) = filter_req.status {
                map.insert("status".to_string(), status);
            }
            
            if let Some(method) = filter_req.method {
                map.insert("method".to_string(), method);
            }
            
            if !map.is_empty() {
                filters = Some(map);
            }
        }

        let payments = self.service.get_all_payments(filters);
        
        ApiResponse {
            success: true,
            data: Some(payments),
            message: None,
        }
    }

    pub fn delete_payment(&self, payment_id: &str) -> ApiResponse<()> {
        match self.service.delete_payment(payment_id.to_string()) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(()),
                message: Some("Pembayaran berhasil dihapus".to_string()),
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }
}

// Updated endpoints to return correct status codes for invalid scenarios.

#[post("/create", format = "json", data = "<request>")]
pub fn create_payment_endpoint(
    request: Json<CreatePaymentRequest>,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    let response = controller.create_payment(request.into_inner());
    if response.success {
        (Status::Ok, Json(response))
    } else {
        (Status::BadRequest, Json(response))
    }
}

#[put("/update_status", format = "json", data = "<request>")]
pub fn update_payment_status_endpoint(
    request: Json<UpdatePaymentStatusRequest>,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    let response = controller.update_payment_status(request.into_inner());
    if response.success {
        (Status::Ok, Json(response))
    } else {
        (Status::BadRequest, Json(response))
    }
}

#[put("/add_installment", format = "json", data = "<request>")]
pub fn add_installment_endpoint(
    request: Json<AddInstallmentRequest>,
    controller: &State<PaymentController>,
) -> Json<ApiResponse<Payment>> {
    Json(controller.add_installment(request.into_inner()))
}

#[get("/<payment_id>")]
pub fn get_payment_endpoint(
    payment_id: String,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    let response = controller.get_payment(&payment_id);
    if response.success {
        (Status::Ok, Json(response))
    } else {
        (Status::NotFound, Json(response))
    }
}

#[get("/transaction/<transaction_id>")]
pub fn get_payment_by_transaction_endpoint(
    transaction_id: String,
    controller: &State<PaymentController>,
) -> Json<ApiResponse<Payment>> {
    Json(controller.get_payment_by_transaction(&transaction_id))
}

#[get("/all?<filter>")]
pub fn get_all_payments_endpoint(
    filter: Option<&str>,
    controller: &State<PaymentController>,
) -> Json<ApiResponse<Vec<Payment>>> {
    let filter_request = filter
        .and_then(|f| serde_json::from_str::<PaymentFilterRequest>(f).ok());
    Json(controller.get_all_payments(filter_request))
}

#[delete("/<payment_id>")]
pub fn delete_payment_endpoint(
    payment_id: String,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<()>>) {
    let response = controller.delete_payment(&payment_id);
    if response.success {
        (Status::Ok, Json(response))
    } else {
        (Status::NotFound, Json(response))
    }
}

#[catch(404)]
pub fn not_found_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        data: None,
        message: Some("Resource not found".to_string()),
    })
}

#[catch(400)]
pub fn bad_request_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        data: None,
        message: Some("Bad request".to_string()),
    })
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_payment_endpoint,
        update_payment_status_endpoint,
        add_installment_endpoint,
        get_payment_endpoint,
        get_payment_by_transaction_endpoint,
        get_all_payments_endpoint,
        delete_payment_endpoint,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::manajemen_pembayaran::repository::payment_repository_impl::PaymentRepositoryImpl;
    use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;
    use crate::manajemen_pembayaran::patterns::observer::{PaymentSubject, TransactionObserver, PaymentObserver};
    use crate::manajemen_pembayaran::service::payment_service_impl::PaymentServiceImpl;
    use crate::manajemen_pembayaran::service::payment_service::PaymentService;

    fn setup_controller() -> PaymentController {
        let repository: Arc<dyn PaymentRepository> = Arc::new(PaymentRepositoryImpl::new());
        
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository, subject));
        
        PaymentController::new(service)
    }

    #[test]
    fn test_create_payment() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let response = controller.create_payment(request);
        
        assert!(response.success);
        assert!(response.data.is_some());
        let payment = response.data.unwrap();
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 1000.0);
        assert_eq!(payment.method, PaymentMethod::Cash);
    }

    #[test]
    fn test_invalid_payment_method() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "INVALID".to_string(),
        };
        
        let response = controller.create_payment(request);
        
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.message.is_some());
        assert_eq!(response.message.unwrap(), "Metode pembayaran tidak valid");
    }

    #[test]
    fn test_update_payment_status() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let create_request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let update_request = UpdatePaymentStatusRequest {
            payment_id: payment.id.clone(),
            new_status: "CICILAN".to_string(),
            additional_amount: Some(500.0),
        };
        
        let update_response = controller.update_payment_status(update_request);
        
        assert!(update_response.success);
        assert!(update_response.data.is_some());
        let updated_payment = update_response.data.unwrap();
        assert_eq!(updated_payment.status, PaymentStatus::Installment);
        assert_eq!(updated_payment.installments.len(), 1);
        assert_eq!(updated_payment.installments[0].amount, 500.0);
    }

    #[test]
    fn test_get_payment() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let create_request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let get_response = controller.get_payment(&payment.id);
        
        assert!(get_response.success);
        assert!(get_response.data.is_some());
        assert_eq!(get_response.data.unwrap().id, payment.id);
        
        let get_by_tx_response = controller.get_payment_by_transaction(&transaction_id);
        
        assert!(get_by_tx_response.success);
        assert!(get_by_tx_response.data.is_some());
        assert_eq!(get_by_tx_response.data.unwrap().transaction_id, transaction_id);
    }

    #[test]
    fn test_add_installment() {
        let controller = setup_controller();
        
        let create_request = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let update_request = UpdatePaymentStatusRequest {
            payment_id: payment.id.clone(),
            new_status: "CICILAN".to_string(),
            additional_amount: None,
        };
        
        controller.update_payment_status(update_request);
        
        let add_request = AddInstallmentRequest {
            payment_id: payment.id.clone(),
            amount: 500.0,
        };
        
        let add_response = controller.add_installment(add_request);
        
        assert!(add_response.success);
        assert!(add_response.data.is_some());
        assert_eq!(add_response.data.unwrap().installments.len(), 1);
    }

    #[test]
    fn test_delete_payment() {
        let controller = setup_controller();
        
        let create_request = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let update_request = UpdatePaymentStatusRequest {
            payment_id: payment.id.clone(),
            new_status: "CICILAN".to_string(),
            additional_amount: None,
        };
        
        controller.update_payment_status(update_request);
        
        let delete_response = controller.delete_payment(&payment.id);
        
        assert!(delete_response.success);
        assert!(delete_response.data.is_some());
        
        let get_response = controller.get_payment(&payment.id);
        assert!(!get_response.success);
    }

    #[test]
    fn test_get_all_payments() {
        let controller = setup_controller();

        let request1 = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        controller.create_payment(request1);

        let request2 = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 2000.0,
            method: "CREDIT_CARD".to_string(),
        };
        controller.create_payment(request2);

        let response = controller.get_all_payments(None);
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().len(), 2);

        let filter_request = PaymentFilterRequest {
            status: None,
            method: Some("CASH".to_string()),
        };
        let response = controller.get_all_payments(Some(filter_request));
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().len(), 1);
    }

    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};

    fn setup_rocket() -> rocket::Rocket<rocket::Build> {
        let controller = setup_controller();
        rocket::build()
            .manage(controller)
            .mount("/payments", get_routes())
            .register("/", catchers![not_found_catcher, bad_request_catcher])
    }

    #[test]
    fn test_create_payment_with_invalid_method() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let request_body = serde_json::json!({
            "transaction_id": "TRX-INVALID",
            "amount": 1000.0,
            "method": "INVALID_METHOD"
        });

        let response = client
            .post("/payments/create")
            .header(ContentType::JSON)
            .body(request_body.to_string())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Metode pembayaran tidak valid");
        } else {
            panic!("Response body is None");
        }
    }

    #[test]
    fn test_get_payment_not_found() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let response = client.get("/payments/unknown_id").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Pembayaran dengan ID unknown_id tidak ditemukan");
        } else {
            panic!("Response body is None");
        }
    }

    #[test]
    fn test_delete_payment_not_found() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let response = client.delete("/payments/unknown_id").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Pembayaran dengan ID unknown_id tidak ditemukan");
        } else {
            panic!("Response body is None");
        }
    }

    #[test]
    fn test_update_payment_status_invalid_status() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let request_body = serde_json::json!({
            "payment_id": "valid_id",
            "new_status": "INVALID_STATUS",
            "additional_amount": 500.0
        });

        let response = client
            .put("/payments/update_status")
            .header(ContentType::JSON)
            .body(request_body.to_string())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Status pembayaran tidak valid");
        } else {
            panic!("Response body is None");
        }
    }
}