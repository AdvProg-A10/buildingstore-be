use super::super::model::Produk;
use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json::json;

#[test]
fn test_remove_produk_from_list() {
    let produk1 = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    let produk2 = Produk::new(
        "Cat Tembok".to_string(),
        "Material".to_string(),
        150_000.0,
        50,
        Some("Cat tembok anti air".to_string()),
    );
    
    let produk3 = Produk::new(
        "Smartphone".to_string(),
        "Elektronik".to_string(),
        8_000_000.0,
        20,
        Some("Smartphone dengan kamera 108MP".to_string()),
    );
    
    let mut produk_list = vec![produk1, produk2, produk3];
    
    // Remove produk dari list (misalnya berdasarkan nama)
    produk_list.retain(|p| p.nama != "Cat Tembok");
    
    assert_eq!(produk_list.len(), 2);
    assert_eq!(produk_list[0].nama, "Laptop Gaming");
    assert_eq!(produk_list[1].nama, "Smartphone");
}

// Controller tests for DELETE operations
#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_delete_produk_controller() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // First create a product to delete
    let create_response = client.post("/produk")
        .json(&json!({
            "nama": "Product to Delete",
            "kategori": "Test",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "This product will be deleted"
        }))
        .dispatch();
    
    assert_eq!(create_response.status(), Status::Ok);
    let body = create_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(body["success"], json!(true), "Failed to create test product");
    
    let product_id = body["data"]["id"].as_i64().expect("valid product id");
    
    // Now delete the product
    let delete_response = client.delete(format!("/produk/{}", product_id))
        .dispatch();
    
    assert_eq!(delete_response.status(), Status::Ok);
    let delete_body = delete_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(delete_body["success"], json!(true), "Failed to delete product");
    assert!(delete_body["message"].as_str().unwrap().contains("berhasil dihapus"));
    
    // Try to get the deleted product to verify it's gone
    let get_response = client.get(format!("/produk/{}", product_id)).dispatch();
    let get_body = get_response.into_json::<serde_json::Value>().expect("valid json response");
    assert_eq!(get_body["success"], json!(false), "Product should not exist after deletion");
    assert!(get_body["message"].as_str().unwrap().contains("tidak ditemukan"));
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_delete_nonexistent_produk() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // Try to delete a product with an ID that doesn't exist
    let delete_response = client.delete("/produk/99999").dispatch();
    
    assert_eq!(delete_response.status(), Status::Ok);
    let delete_body = delete_response.into_json::<serde_json::Value>().expect("valid json response");
    
    // The response should indicate failure
    assert_eq!(delete_body["success"], json!(false));
    assert!(delete_body["message"].as_str().unwrap().contains("tidak ditemukan"));
}

#[test]
#[ignore] // Add this attribute to skip in regular runs if database setup is required
fn test_delete_multiple_produk() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    
    // Create multiple products
    let product_ids = (0..3).map(|i| {
        let create_response = client.post("/produk")
            .json(&json!({
                "nama": format!("Bulk Delete Test {}", i),
                "kategori": "Test",
                "harga": 1000.0 * (i as f64 + 1.0),
                "stok": 10 * (i + 1),
                "deskripsi": format!("Test product {} for bulk deletion", i)
            }))
            .dispatch();
        
        let body = create_response.into_json::<serde_json::Value>().expect("valid json response");
        body["data"]["id"].as_i64().expect("valid product id")
    }).collect::<Vec<_>>();
    
    // Delete each product and verify
    for id in product_ids {
        let delete_response = client.delete(format!("/produk/{}", id)).dispatch();
        assert_eq!(delete_response.status(), Status::Ok);
        
        let delete_body = delete_response.into_json::<serde_json::Value>().expect("valid json response");
        assert_eq!(delete_body["success"], json!(true));
        
        // Verify deletion
        let get_response = client.get(format!("/produk/{}", id)).dispatch();
        let get_body = get_response.into_json::<serde_json::Value>().expect("valid json response");
        assert_eq!(get_body["success"], json!(false));
    }
}