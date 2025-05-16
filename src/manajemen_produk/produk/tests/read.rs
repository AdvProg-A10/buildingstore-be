use super::super::model::Produk;
use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json::json;

fn setup_test_products() -> Vec<Produk> {
    vec![
        Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        ),
        Produk::new(
            "Cat Tembok".to_string(),
            "Material".to_string(),
            150_000.0,
            50,
            Some("Cat tembok anti air".to_string()),
        ),
        Produk::new(
            "Smartphone".to_string(),
            "Elektronik".to_string(),
            8_000_000.0,
            20,
            Some("Smartphone dengan kamera 108MP".to_string()),
        ),
    ]
}

#[test]
fn test_filter_produk_by_kategori() {
    let produk_list = setup_test_products();
    
    // Filter produk elektronik
    let elektronik_list: Vec<&Produk> = produk_list.iter()
        .filter(|p| p.kategori == "Elektronik")
        .collect();
    
    assert_eq!(elektronik_list.len(), 2);
    assert_eq!(elektronik_list[0].nama, "Laptop Gaming");
    assert_eq!(elektronik_list[1].nama, "Smartphone");
}

#[test]
fn test_sort_produk_by_harga() {
    let mut produk_list = setup_test_products();
    
    // Sort by harga (ascending)
    produk_list.sort_by(|a, b| a.harga.partial_cmp(&b.harga).unwrap());
    
    assert_eq!(produk_list[0].nama, "Cat Tembok");
    assert_eq!(produk_list[1].nama, "Smartphone");
    assert_eq!(produk_list[2].nama, "Laptop Gaming");
}

#[test]
fn test_find_produk_by_nama() {
    let produk_list = setup_test_products();
    
    let found_produk = produk_list.iter()
        .find(|p| p.nama == "Smartphone");
    
    assert!(found_produk.is_some());
    let produk = found_produk.unwrap();
    assert_eq!(produk.kategori, "Elektronik");
    assert_eq!(produk.harga, 8_000_000.0);
}

// Controller tests for READ operations
#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_list_produk_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // First add a test product
    client.post("/produk")
        .json(&json!({
            "nama": "Test Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "Test product description"
        }))
        .dispatch();
    
    // Now test list endpoint
    let response = client.get("/produk").dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
    assert!(body["data"].is_array());
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_detail_produk_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // First add a test product
    let create_response = client.post("/produk")
        .json(&json!({
            "nama": "Detail Test Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "Test product for detail view"
        }))
        .dispatch();
    
    let body = create_response.into_json::<serde_json::Value>().expect("valid json response");
    let product_id = body["data"]["id"].as_i64().expect("valid product id");
    
    // Now test detail endpoint
    let detail_response = client.get(format!("/produk/{}", product_id)).dispatch();
    
    assert_eq!(detail_response.status(), Status::Ok);
    let detail_body = detail_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(detail_body["success"], json!(true));
    assert_eq!(detail_body["data"]["nama"], json!("Detail Test Product"));
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_filter_produk_by_kategori_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // Add test products with different categories
    client.post("/produk")
        .json(&json!({
            "nama": "Test Elektronik 1",
            "kategori": "Elektronik",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "Test elektronik 1"
        }))
        .dispatch();
        
    client.post("/produk")
        .json(&json!({
            "nama": "Test Material 1",
            "kategori": "Material",
            "harga": 2000.0,
            "stok": 20,
            "deskripsi": "Test material 1"
        }))
        .dispatch();
    
    // Test filter by category
    let response = client.get("/produk/kategori/Elektronik").dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
    
    // Check that all returned products are in the Elektronik category
    for product in body["data"].as_array().expect("data is an array") {
        assert_eq!(product["kategori"], json!("Elektronik"));
    }
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_filter_produk_by_price_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // Add test products with different prices
    client.post("/produk")
        .json(&json!({
            "nama": "Cheap Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "Cheap test product"
        }))
        .dispatch();
        
    client.post("/produk")
        .json(&json!({
            "nama": "Expensive Product",
            "kategori": "Test",
            "harga": 5000.0,
            "stok": 5,
            "deskripsi": "Expensive test product"
        }))
        .dispatch();
    
    // Test price filter
    let response = client.get("/produk/harga?min=500&max=2000").dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
    
    // Check that all returned products are in the price range
    for product in body["data"].as_array().expect("data is an array") {
        let price = product["harga"].as_f64().expect("price is a number");
        assert!(price >= 500.0 && price <= 2000.0);
    }
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_filter_produk_by_stock_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // Add test products with different stock levels
    client.post("/produk")
        .json(&json!({
            "nama": "Low Stock Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 3,
            "deskripsi": "Low stock test product"
        }))
        .dispatch();
        
    client.post("/produk")
        .json(&json!({
            "nama": "High Stock Product",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 15,
            "deskripsi": "High stock test product"
        }))
        .dispatch();
    
    // Test stock filter (minimum 10)
    let response = client.get("/produk/stok?min_stok=10").dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true));
    
    // Check that all returned products have at least the minimum stock
    for product in body["data"].as_array().expect("data is an array") {
        let stock = product["stok"].as_u64().expect("stock is a number");
        assert!(stock >= 10);
    }
}