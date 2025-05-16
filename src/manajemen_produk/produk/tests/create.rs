use super::super::model::{Produk, validate_produk};
use crate::manajemen_produk::produk::model::ProdukBuilder;
use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json::json;
use crate::manajemen_produk::produk::tests::create::serde_json::json;

#[test]
fn test_create_produk_baru() {
    let produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );

    assert_eq!(produk.nama, "Laptop Gaming");
    assert_eq!(produk.kategori, "Elektronik");
    assert_eq!(produk.harga, 15_000_000.0);
    assert_eq!(produk.stok, 10);
    assert_eq!(produk.deskripsi, Some("Laptop dengan RTX 4060".to_string()));
}

#[test]
fn test_create_produk_without_deskripsi() {
    let produk = Produk::new(
        "Cat Tembok".to_string(),
        "Material".to_string(),
        150_000.0,
        50,
        None,
    );

    assert_eq!(produk.nama, "Cat Tembok");
    assert_eq!(produk.kategori, "Material");
    assert_eq!(produk.harga, 150_000.0);
    assert_eq!(produk.stok, 50);
    assert_eq!(produk.deskripsi, None);
}

#[test]
fn test_validasi_produk() {
    // Testing valid product
    let result = validate_produk(
        &"Laptop Gaming".to_string(),  // Add reference
        &"Elektronik".to_string(),     // Add reference
        15_000_000.0,
        10,
        &Some("Laptop dengan RTX 4060".to_string()),  // Add reference
    );
    assert!(result.is_ok());
    
    // Testing invalid product (empty name)
    let result = validate_produk(
        &"".to_string(),               // Add reference
        &"Elektronik".to_string(),     // Add reference
        15_000_000.0,
        10,
        &Some("Laptop dengan RTX 4060".to_string()),  // Add reference
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), vec!["Nama produk tidak boleh kosong"]);
    
    // Testing invalid product (negative price)
    let result = validate_produk(
        &"Laptop Gaming".to_string(),  // Add reference
        &"Elektronik".to_string(),     // Add reference
        -5000.0,
        10,
        &Some("Laptop dengan RTX 4060".to_string()),  // Add reference
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), vec!["Harga tidak boleh negatif"]);
}

#[test]
fn test_create_with_validation() {
    // Testing valid product creation
    let result = Produk::create_with_validation(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(result.is_ok());
    
    // Testing invalid product creation (empty name)
    let result = Produk::create_with_validation(
        "".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), vec!["Nama produk tidak boleh kosong"]);
}

#[test]
fn test_produk_builder() {
    // Using the builder pattern
    let produk_result = ProdukBuilder::new("Laptop Gaming".to_string(), "Elektronik".to_string())
        .harga(15_000_000.0)
        .stok(10)
        .deskripsi("Laptop dengan RTX 4060".to_string())
        .build();
    
    assert!(produk_result.is_ok());
    let produk = produk_result.unwrap();
    
    assert_eq!(produk.nama, "Laptop Gaming");
    assert_eq!(produk.kategori, "Elektronik");
    assert_eq!(produk.harga, 15_000_000.0);
    assert_eq!(produk.stok, 10);
    assert_eq!(produk.deskripsi, Some("Laptop dengan RTX 4060".to_string()));
}

#[test]
fn test_builder_validation() {
    // Test validation with empty name
    let produk_result = ProdukBuilder::new("".to_string(), "Elektronik".to_string())
        .harga(15_000_000.0)
        .stok(10)
        .build();
    
    assert!(produk_result.is_err());
    assert_eq!(produk_result.unwrap_err(), vec!["Nama produk tidak boleh kosong"]);
    
    // Test validation with negative price
    let produk_result = ProdukBuilder::new("Laptop Gaming".to_string(), "Elektronik".to_string())
        .harga(-5000.0)
        .stok(10)
        .build();
    
    assert!(produk_result.is_err());
    assert_eq!(produk_result.unwrap_err(), vec!["Harga tidak boleh negatif"]);
}

#[test]
fn test_produk_factory_methods() {
    // Using the factory method for laptops
    let laptop = Produk::create_laptop(
        "Gaming Laptop".to_string(),
        12_000_000.0,
        5,
        Some("High-performance gaming laptop".to_string())
    );
    
    assert_eq!(laptop.nama, "Gaming Laptop");
    assert_eq!(laptop.kategori, "Elektronik");
    assert_eq!(laptop.harga, 12_000_000.0);
    assert_eq!(laptop.stok, 5);
    
    // Using the factory method for building materials
    let material = Produk::create_building_material(
        "Semen".to_string(),
        75_000.0,
        100,
        Some("Semen tahan air".to_string())
    );
    
    assert_eq!(material.nama, "Semen");
    assert_eq!(material.kategori, "Material");
    assert_eq!(material.harga, 75_000.0);
    assert_eq!(material.stok, 100);
}

// Controller tests for CREATE operations
#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_tambah_produk_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.post("/produk")
        .json(&json!({
            "nama": "Laptop Test",
            "kategori": "Elektronik",
            "harga": 12000000.0,
            "stok": 5,
            "deskripsi": "Laptop untuk testing"
        }))
        .dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
    assert!(body["data"]["id"].is_number());
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_tambah_produk_with_factory_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.post("/produk/factory")
        .json(&json!({
            "nama": "Laptop Factory Test",
            "kategori": "Elektronik",
            "harga": 12000000.0,
            "stok": 5,
            "deskripsi": "Laptop dibuat melalui factory"
        }))
        .dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_tambah_produk_from_template_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.post("/produk/template/laptop_gaming")
        .json(&json!({
            "nama": "Template Laptop",
            "harga": 10000000.0,
            "stok": 8,
            "deskripsi": "Laptop dari template"
        }))
        .dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_clone_produk_with_price_controller() {
    // First create a product to clone
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let create_response = client.post("/produk")
        .json(&json!({
            "nama": "Product to Clone",
            "kategori": "Elektronik",
            "harga": 5000000.0,
            "stok": 10,
            "deskripsi": "Original product"
        }))
        .dispatch();
    
    let body = create_response.into_json::<serde_json::Value>().expect("valid json response");
    let product_id = body["data"]["id"].as_i64().expect("valid product id");
    
    // Now clone the product with a new price
    let clone_response = client.post(format!("/produk/{}/clone_with_price", product_id))
        .json(&json!({
            "nama": "Cloned Product",
            "harga": 4500000.0,
            "kategori": "Elektronik",
            "stok": 10,
            "deskripsi": "Cloned product"
        }))
        .dispatch();
    
    assert_eq!(clone_response.status(), Status::Ok);
    let clone_body = clone_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(clone_body["success"], json!(true));
    assert_eq!(clone_body["data"]["harga"], json!(4500000.0));
}