use super::super::model::Produk;
use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json::json;

#[test]
fn test_update_produk_harga() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    // Update harga
    produk.set_harga(14_500_000.0).unwrap();
    
    assert_eq!(produk.harga, 14_500_000.0);
}

#[test]
fn test_update_produk_stok() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    // Update stok
    produk.set_stok(5);
    
    assert_eq!(produk.stok, 5);
}

#[test]
fn test_update_produk_deskripsi() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    // Update deskripsi
    produk.deskripsi = Some("Laptop Gaming dengan RTX 4060 dan RAM 32GB".to_string());
    
    assert_eq!(produk.deskripsi, Some("Laptop Gaming dengan RTX 4060 dan RAM 32GB".to_string()));
}

// Controller tests for UPDATE operations
#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_update_produk_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // First create a product to update
    let create_response = client.post("/produk")
        .json(&json!({
            "nama": "Original Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "Original description"
        }))
        .dispatch();
    
    let body = create_response.into_json::<serde_json::Value>().expect("valid json response");
    let product_id = body["data"]["id"].as_i64().expect("valid product id");
    
    // Now update the product
    let update_response = client.put(format!("/produk/{}", product_id))
        .json(&json!({
            "nama": "Updated Product",
            "kategori": "TestUpdated",
            "harga": 1500.0,
            "stok": 15,
            "deskripsi": "Updated description"
        }))
        .dispatch();
    
    assert_eq!(update_response.status(), Status::Ok);
    let update_body = update_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(update_body["success"], json!(true));
    assert_eq!(update_body["data"]["nama"], json!("Updated Product"));
    assert_eq!(update_body["data"]["harga"], json!(1500.0));
    
    // Verify the update was persisted
    let detail_response = client.get(format!("/produk/{}", product_id)).dispatch();
    let detail_body = detail_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(detail_body["data"]["nama"], json!("Updated Product"));
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_update_stock_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // First create a product
    let create_response = client.post("/produk")
        .json(&json!({
            "nama": "Stock Test Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "Test product for stock update"
        }))
        .dispatch();
    
    let body = create_response.into_json::<serde_json::Value>().expect("valid json response");
    let product_id = body["data"]["id"].as_i64().expect("valid product id");
    
    // Now update just the stock
    let update_response = client.post(format!("/produk/{}/update_stock", product_id))
        .json(&json!(20))
        .dispatch();
    
    assert_eq!(update_response.status(), Status::Ok);
    let update_body = update_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(update_body["success"], json!(true));
    
    // Verify the stock was updated
    let detail_response = client.get(format!("/produk/{}", product_id)).dispatch();
    let detail_body = detail_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(detail_body["data"]["stok"], json!(20));
}