#![cfg(test)]

use rocket::{local::asynchronous::Client, http::{Status, ContentType}};
use std::sync::Arc;
use chrono::Utc;

use mockall::{automock, predicate::*};

use crate::manajemen_supplier::main::{
    model::supplier::Supplier,
    service::supplier_service::SupplierService,
    controller::supplier_controller::{supplier_routes, SupplierRequest},
};

#[automock]
pub trait SupplierServiceMock: Send + Sync {
    fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String>;
    fn get_supplier(&self, id: &str) -> Option<Supplier>;
    fn update_supplier(&self, supplier: Supplier) -> Result<(), String>;
    fn delete_supplier(&self, id: &str) -> Result<(), String>;
}

impl SupplierService for MockSupplierServiceMock {
    fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
        SupplierServiceMock::save_supplier(self, supplier)
    }

    fn get_supplier(&self, id: &str) -> Option<Supplier> {
        SupplierServiceMock::get_supplier(self, id)
    }

    fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
        SupplierServiceMock::update_supplier(self, supplier)
    }

    fn delete_supplier(&self, id: &str) -> Result<(), String> {
        SupplierServiceMock::delete_supplier(self, id)
    }
}

impl SupplierRequest {
    pub fn new(name: &str, jenis_barang: &str, jumlah_barang: i32, resi: &str) -> Self {
        Self {
            name: name.to_string(),
            jenis_barang: jenis_barang.to_string(),
            jumlah_barang,
            resi: resi.to_string(),
        }
    }
}

// --- Helpers ---

fn create_supplier() -> Supplier {
    Supplier {
        id: "test-id".to_string(),
        name: "Test Name".to_string(),
        jenis_barang: "Type".to_string(),
        jumlah_barang: 20,
        resi: "ResiTest".to_string(),
        updated_at: Utc::now(),
    }
}

fn create_supplier_request(s: &Supplier) -> SupplierRequest {
    SupplierRequest::new(&s.name, &s.jenis_barang, s.jumlah_barang, &s.resi)
}

async fn build_test_client_with_mock(
    setup_mock: impl FnOnce(&mut MockSupplierServiceMock)
) -> Client {
    let mut mock = MockSupplierServiceMock::new();
    setup_mock(&mut mock);
    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    Client::tracked(rocket).await.unwrap()
}

async fn post_json<'a>(client: &'a Client, url: &'a str, req: &'a SupplierRequest) -> rocket::local::asynchronous::LocalResponse<'a> {
    client.post(url)
        .header(ContentType::JSON)
        .body(serde_json::to_string(req).unwrap())
        .dispatch().await
}

async fn put_json<'a>(client: &'a Client, url: &'a str, req: &'a SupplierRequest) -> rocket::local::asynchronous::LocalResponse<'a> {
    client.put(url)
        .header(ContentType::JSON)
        .body(serde_json::to_string(req).unwrap())
        .dispatch().await
}

fn assert_failure_response(res: &str, message_substr: &str) {
    assert!(res.contains(message_substr));
    assert!(res.contains("\"success\":false"));
}

// --- Tests ---

#[rocket::async_test]
async fn test_get_supplier_found() {
    let supplier = create_supplier();

    let client = build_test_client_with_mock(|mock| {
        mock.expect_get_supplier()
            .with(eq("test-id"))
            .return_const(Some(supplier.clone()));
    }).await;

    let resp = client.get("/suppliers/test-id").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body = resp.into_string().await.unwrap();
    assert!(body.contains("Supplier found"));
    assert!(body.contains(&supplier.name));
}

#[rocket::async_test]
async fn test_get_supplier_not_found() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_get_supplier()
            .with(eq("unknown-id"))
            .return_const(None);
    }).await;

    let resp = client.get("/suppliers/unknown-id").dispatch().await;
    assert_eq!(resp.status(), Status::NotFound);
    let body = resp.into_string().await.unwrap();
    assert!(body.contains("not found"));
}

#[rocket::async_test]
async fn test_save_supplier_success() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_save_supplier()
            .returning(|sup| Ok(sup));
    }).await;

    let req = SupplierRequest::new("New", "Type", 10, "123");
    let resp = post_json(&client, "/suppliers", &req).await;

    assert_eq!(resp.status(), Status::Created);
    let body = resp.into_string().await.unwrap();
    assert!(body.contains("Supplier created successfully"));
}

#[rocket::async_test]
async fn test_update_supplier_success() {
    let supplier = create_supplier();

    let client = build_test_client_with_mock(|mock| {
        mock.expect_get_supplier()
            .with(eq("test-id"))
            .return_const(Some(supplier.clone()));
        mock.expect_update_supplier()
            .returning(|_| Ok(()));
    }).await;

    let req = create_supplier_request(&supplier);
    let resp = put_json(&client, "/suppliers/test-id", &req).await;

    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().await.unwrap();
    assert!(body.contains("Supplier updated successfully"));
}

#[rocket::async_test]
async fn test_delete_supplier_success() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_delete_supplier()
            .with(eq("test-id"))
            .returning(|_| Ok(()));
    }).await;

    let resp = client.delete("/suppliers/test-id").dispatch().await;
    assert_eq!(resp.status(), Status::NoContent);
}

#[rocket::async_test]
async fn test_save_supplier_failure() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_save_supplier()
            .returning(|_| Err("Failed to save".to_string()));
    }).await;

    let req = SupplierRequest::new("New", "Type", 10, "123");
    let resp = post_json(&client, "/suppliers", &req).await;

    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().await.unwrap();
    assert_failure_response(&body, "Failed to create supplier");
}

#[rocket::async_test]
async fn test_update_supplier_not_found() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_update_supplier()
            .returning(|_| Err("No such supplier".to_string()));
    }).await;

    let req = SupplierRequest::new("None", "None", 0, "");
    let resp = put_json(&client, "/suppliers/missing-id", &req).await;

    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().await.unwrap();
    assert_failure_response(&body, "Failed to update supplier: No such supplier");
}

#[rocket::async_test]
async fn test_update_supplier_get_updated_failed() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_update_supplier().returning(|_| Ok(()));
        mock.expect_get_supplier()
            .with(eq("test-id"))
            .return_const(None);
    }).await;

    let req = SupplierRequest::new("Test", "Type", 10, "A");
    let resp = put_json(&client, "/suppliers/test-id", &req).await;

    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().await.unwrap();
    assert_failure_response(&body, "Failed to fetch updated supplier");
}

#[rocket::async_test]
async fn test_update_supplier_failure() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_update_supplier()
            .returning(|_| Err("Failed update".to_string()));
    }).await;

    let supplier = create_supplier();
    let req = create_supplier_request(&supplier);

    let resp = put_json(&client, "/suppliers/test-id", &req).await;

    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().await.unwrap();
    assert_failure_response(&body, "Failed to update supplier: Failed update");
}

#[rocket::async_test]
async fn test_delete_supplier_not_found() {
    let client = build_test_client_with_mock(|mock| {
        mock.expect_delete_supplier()
            .with(eq("missing-id"))
            .returning(|_| Err("Supplier not found".to_string()));
    }).await;

    let resp = client.delete("/suppliers/missing-id").dispatch().await;
    assert_eq!(resp.status(), Status::NotFound);
    let body = resp.into_string().await.unwrap();
    assert_failure_response(&body, "Failed to delete supplier");
}

#[rocket::async_test]
async fn test_malformed_post_request() {
    let client = build_test_client_with_mock(|_| {}).await;

    let resp = client.post("/suppliers")
        .header(ContentType::JSON)
        .body("{bad json")
        .dispatch().await;

    assert_eq!(resp.status(), Status::BadRequest);
}

#[rocket::async_test]
async fn test_malformed_put_request() {
    let client = build_test_client_with_mock(|_| {}).await;

    let resp = client.put("/suppliers/test-id")
        .header(ContentType::JSON)
        .body("{bad json")
        .dispatch().await;

    assert_eq!(resp.status(), Status::BadRequest);
}