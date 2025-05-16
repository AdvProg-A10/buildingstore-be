use rocket::{State, get, put, delete, post, routes};
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::response::status::{Created, NotFound, NoContent};
use std::sync::Arc;

use crate::manajemen_supplier::main::{
    model::supplier::Supplier,
    service::supplier_service::SupplierService,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SupplierRequest {
    pub name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub resi: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SupplierResponse {
    pub id: String,
    pub name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub resi: String,
    pub updated_at: String,
}

impl From<Supplier> for SupplierResponse {
    fn from(supplier: Supplier) -> Self {
        Self {
            id: supplier.id,
            name: supplier.name,
            jenis_barang: supplier.jenis_barang,
            jumlah_barang: supplier.jumlah_barang,
            resi: supplier.resi,
            updated_at: supplier.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Clone)]
pub struct SupplierController {
    pub service: Arc<dyn SupplierService>,
}

impl SupplierController {
    pub fn new(service: Arc<dyn SupplierService>) -> Self {
        Self { service }
    }
}

#[post("/suppliers", format = "json", data = "<request>")]
pub async fn save_supplier(
    request: Json<SupplierRequest>,
    service: &State<Arc<dyn SupplierService>>,
) -> Result<Created<Json<ApiResponse<SupplierResponse>>>, Json<ApiResponse<String>>> {
    let supplier = Supplier {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name.clone(),
        jenis_barang: request.jenis_barang.clone(),
        jumlah_barang: request.jumlah_barang,
        resi: request.resi.clone(),
        updated_at: chrono::Utc::now(),
    };

    match service.save_supplier(supplier) {
        Ok(saved) => {
            let response = ApiResponse {
                success: true,
                message: "Supplier created successfully".to_string(),
                data: Some(SupplierResponse::from(saved)),
            };
            Ok(Created::new("/suppliers").body(Json(response)))
        }
        Err(e) => Err(Json(ApiResponse {
            success: false,
            message: format!("Failed to create supplier: {}", e),
            data: None,
        }))
    }
}

#[get("/suppliers/<id>")]
pub async fn get_supplier(
    id: String,
    service: &State<Arc<dyn SupplierService>>,
) -> Result<Json<ApiResponse<SupplierResponse>>, NotFound<Json<ApiResponse<String>>>> {
    match service.get_supplier(&id) {
        Some(supplier) => Ok(Json(ApiResponse {
            success: true,
            message: "Supplier found".to_string(),
            data: Some(SupplierResponse::from(supplier)),
        })),
        None => Err(NotFound(Json(ApiResponse {
            success: false,
            message: format!("Supplier with id {} not found", id),
            data: None,
        }))),
    }
}

#[put("/suppliers/<id>", format = "json", data = "<request>")]
pub async fn update_supplier(
    service: &State<Arc<dyn SupplierService>>,
    id: String,
    request: Json<SupplierRequest>,
) -> Result<Json<ApiResponse<SupplierResponse>>, Json<ApiResponse<String>>> {
    let supplier = Supplier {
        id: id.clone(),
        name: request.name.clone(),
        jenis_barang: request.jenis_barang.clone(),
        jumlah_barang: request.jumlah_barang,
        resi: request.resi.clone(),
        updated_at: chrono::Utc::now(),
    };

    match service.update_supplier(supplier) {
        Ok(()) => {
            if let Some(updated) = service.get_supplier(&id) {
                Ok(Json(ApiResponse {
                    success: true,
                    message: "Supplier updated successfully".to_string(),
                    data: Some(SupplierResponse::from(updated)),
                }))
            } else {
                Err(Json(ApiResponse {
                    success: false,
                    message: "Failed to fetch updated supplier".to_string(),
                    data: None,
                }))
            }
        }
        Err(e) => Err(Json(ApiResponse {
            success: false,
            message: format!("Failed to update supplier: {}", e),
            data: None,
        })),
    }
}

#[delete("/suppliers/<id>")]
pub async fn delete_supplier(
    id: String,
    service: &State<Arc<dyn SupplierService>>,
) -> Result<NoContent, NotFound<Json<ApiResponse<String>>>> {
    match service.delete_supplier(&id) {
        Ok(()) => Ok(NoContent),
        Err(e) => Err(NotFound(Json(ApiResponse {
            success: false,
            message: format!("Failed to delete supplier: {}", e),
            data: None,
        }))),
    }
}

pub fn supplier_routes() -> Vec<rocket::Route> {
    routes![
        save_supplier,
        get_supplier,
        update_supplier,
        delete_supplier,
    ]
}