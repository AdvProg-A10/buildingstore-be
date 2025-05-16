use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket_db_pools::Database;
use serde_json::json;

// Helper function to create rocket instance for testing
fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", crate::manajemen_produk::produk::controller::routes())
        .attach(crate::BuildingStoreDB::init())
}

// CREATE: Invalid input (empty name)
#[test]
#[ignore]
fn test_create_produk_empty_name() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.post("/produk")
        .json(&json!({
            "nama": "",
            "kategori": "Elektronik",
            "harga": 1000.0,
            "stok": 5,
            "deskripsi": "desc"
        }))
        .dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(false));
    assert!(body["message"].as_str().unwrap().contains("Nama produk tidak boleh kosong"));
}

// CREATE: Invalid input (negative price)
#[test]
#[ignore]
fn test_create_produk_negative_price() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.post("/produk")
        .json(&json!({
            "nama": "Test",
            "kategori": "Elektronik",
            "harga": -1000.0,
            "stok": 5,
            "deskripsi": "desc"
        }))
        .dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(false));
    assert!(body["message"].as_str().unwrap().contains("Harga tidak boleh negatif"));
}

// UPDATE: Not found
#[test]
#[ignore]
fn test_update_produk_not_found() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.put("/produk/999999")
        .json(&json!({
            "nama": "Update",
            "kategori": "Elektronik",
            "harga": 1000.0,
            "stok": 5,
            "deskripsi": "desc"
        }))
        .dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(false));
    assert!(body["message"].as_str().unwrap().contains("tidak ditemukan"));
}

// UPDATE: Invalid input (empty name)
#[test]
#[ignore]
fn test_update_produk_invalid() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.put("/produk/1")
        .json(&json!({
            "nama": "",
            "kategori": "Elektronik",
            "harga": 1000.0,
            "stok": 5,
            "deskripsi": "desc"
        }))
        .dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(false));
    assert!(body["message"].as_str().unwrap().contains("Nama produk tidak boleh kosong"));
}

// DELETE: Not found
#[test]
#[ignore]
fn test_delete_produk_not_found() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.delete("/produk/999999").dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(false));
    assert!(body["message"].as_str().unwrap().contains("tidak ditemukan"));
}

// FILTER: By kategori, empty result
#[test]
#[ignore]
fn test_filter_produk_by_kategori_empty() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get("/produk/kategori/NonExistentCategory").dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(true));
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

// FILTER: By price, empty result
#[test]
#[ignore]
fn test_filter_produk_by_price_empty() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get("/produk/harga?min=9999999&max=10000000").dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(true));
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

// FILTER: By stock, empty result
#[test]
#[ignore]
fn test_filter_produk_by_stock_empty() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get("/produk/stok?min_stok=99999").dispatch();
    let body = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(body["success"], json!(true));
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}