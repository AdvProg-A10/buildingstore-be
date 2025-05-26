use rocket::{get, post, patch, delete, put};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::{Any, Pool};
use autometrics::autometrics;

use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::service::transaksi::TransaksiService;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: String,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, data: T) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }
}

impl ErrorResponse {
    pub fn new(error: &str, code: &str) -> Self {
        Self {
            success: false,
            error: error.to_string(),
            code: code.to_string(),
        }
    }
}

#[autometrics]
#[get("/transaksi?<sort>&<filter>&<keyword>&<status>&<id_pelanggan>&<page>&<limit>")]
pub async fn get_all_transaksi(
    db: &State<Pool<Any>>, 
    sort: Option<String>, 
    filter: Option<String>, 
    keyword: Option<String>,
    status: Option<String>,
    id_pelanggan: Option<i32>,  
    page: Option<usize>,
    limit: Option<usize>
) -> Result<Json<ApiResponse<crate::transaksi_penjualan::service::transaksi::TransaksiSearchResult>>, (Status, Json<ErrorResponse>)> {
    let search_params = crate::transaksi_penjualan::service::transaksi::TransaksiSearchParams {
        sort,
        filter,
        keyword,
        status,
        id_pelanggan,
        page,
        limit,
    };

    match TransaksiService::search_transaksi_with_pagination(db.inner().clone(), &search_params).await {
        Ok(result) => Ok(Json(ApiResponse::success("Data transaksi berhasil diambil", result))),
        Err(_) => Err((
            Status::InternalServerError,
            Json(ErrorResponse::new("Gagal mengambil data transaksi", "FETCH_ERROR"))
        ))
    }
}

#[autometrics]
#[post("/transaksi", data = "<request>")]
pub async fn create_transaksi(
    db: &State<Pool<Any>>, 
    request: Json<crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest>
) -> Result<Json<ApiResponse<Transaksi>>, (Status, Json<ErrorResponse>)> {
    if let Err(err_msg) = request.validate() {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new(&err_msg, "VALIDATION_ERROR"))
        ));
    }

    if let Err(err_msg) = TransaksiService::validate_product_stock(&request.detail_transaksi).await {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new(&err_msg, "INSUFFICIENT_STOCK"))
        ));
    }

    match TransaksiService::create_transaksi_with_details(db.inner().clone(), &request).await {
        Ok(new_transaksi) => {
            Ok(Json(ApiResponse::success("Transaksi berhasil dibuat", new_transaksi)))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal membuat transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[get("/transaksi/<id>")]
pub async fn get_transaksi_by_id(
    db: &State<Pool<Any>>, 
    id: i32 
) -> Result<Json<ApiResponse<Transaksi>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::get_transaksi_by_id(db.inner().clone(), id).await {
        Ok(transaksi) => Ok(Json(ApiResponse::success("Data transaksi berhasil diambil", transaksi))),
        Err(_) => Err((
            Status::NotFound,
            Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
        ))
    }
}

#[autometrics]
#[patch("/transaksi/<id>", data = "<transaksi>")]
pub async fn update_transaksi(
    db: &State<Pool<Any>>, 
    id: i32,
    transaksi: Json<Transaksi>
) -> Result<Json<ApiResponse<Transaksi>>, (Status, Json<ErrorResponse>)> {
    if transaksi.id != id {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("ID transaksi tidak sesuai", "INVALID_ID"))
        ));
    }

    match TransaksiService::get_transaksi_by_id(db.inner().clone(), id).await {
        Ok(existing_transaksi) => {
            if !existing_transaksi.can_be_modified() {
                return Err((
                    Status::Forbidden,
                    Json(ErrorResponse::new(
                        "Transaksi tidak dapat dimodifikasi karena sudah selesai atau dibatalkan", 
                        "TRANSACTION_IMMUTABLE"
                    ))
                ));
            }
        }
        Err(_) => {
            return Err((
                Status::NotFound,
                Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
            ));
        }
    }

    match TransaksiService::update_transaksi(db.inner().clone(), &transaksi).await {
        Ok(updated_transaksi) => {
            Ok(Json(ApiResponse::success("Transaksi berhasil diperbarui", updated_transaksi)))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal memperbarui transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[delete("/transaksi/<id>")]
pub async fn delete_transaksi(
    db: &State<Pool<Any>>, 
    id: i32
) -> Result<Json<ApiResponse<String>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::get_transaksi_by_id(db.inner().clone(), id).await {
        Ok(existing_transaksi) => {
            if !existing_transaksi.can_be_cancelled() {
                return Err((
                    Status::Forbidden,
                    Json(ErrorResponse::new(
                        "Transaksi tidak dapat dibatalkan karena sudah selesai", 
                        "CANNOT_CANCEL_COMPLETED"
                    ))
                ));
            }
        }
        Err(_) => {
            return Err((
                Status::NotFound,
                Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
            ));
        }
    }

    match TransaksiService::delete_transaksi(db.inner().clone(), id).await {
        Ok(_) => {
            Ok(Json(ApiResponse::success("Transaksi berhasil dibatalkan", "Deleted".to_string())))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal membatalkan transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[put("/transaksi/<id>/complete")]
pub async fn complete_transaksi(
    db: &State<Pool<Any>>, 
    id: i32
) -> Result<Json<ApiResponse<Transaksi>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::complete_transaksi(db.inner().clone(), id).await {
        Ok(completed_transaksi) => {
            Ok(Json(ApiResponse::success("Transaksi berhasil diselesaikan", completed_transaksi)))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal menyelesaikan transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[put("/transaksi/<id>/cancel")]
pub async fn cancel_transaksi(
    db: &State<Pool<Any>>, 
    id: i32
) -> Result<Json<ApiResponse<Transaksi>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::cancel_transaksi(db.inner().clone(), id).await {
        Ok(cancelled_transaksi) => {
            Ok(Json(ApiResponse::success("Transaksi berhasil dibatalkan", cancelled_transaksi)))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal membatalkan transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[get("/transaksi/<id_transaksi>/detail")]
pub async fn get_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32
) -> Result<Json<ApiResponse<Vec<DetailTransaksi>>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::get_detail_by_transaksi_id(db.inner().clone(), id_transaksi).await {
        Ok(detail_list) => Ok(Json(ApiResponse::success("Detail transaksi berhasil diambil", detail_list))),
        Err(_) => Err((
            Status::NotFound,
            Json(ErrorResponse::new("Detail transaksi tidak ditemukan", "NOT_FOUND"))
        ))
    }
}

#[autometrics]
#[post("/transaksi/<id_transaksi>/detail", data = "<detail>")]
pub async fn add_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32,
    detail: Json<DetailTransaksi>
) -> Result<Json<ApiResponse<DetailTransaksi>>, (Status, Json<ErrorResponse>)> {
    if detail.id_transaksi != id_transaksi {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("ID transaksi tidak sesuai", "INVALID_ID"))
        ));
    }

    match TransaksiService::get_transaksi_by_id(db.inner().clone(), id_transaksi).await {
        Ok(transaksi) => {
            if !transaksi.can_be_modified() {
                return Err((
                    Status::Forbidden,
                    Json(ErrorResponse::new(
                        "Transaksi tidak dapat dimodifikasi karena sudah selesai atau dibatalkan",
                        "TRANSACTION_IMMUTABLE"
                    ))
                ));
            }
        }
        Err(_) => {
            return Err((
                Status::NotFound,
                Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
            ));
        }
    }

    match TransaksiService::add_detail_transaksi(db.inner().clone(), &detail).await {
        Ok(new_detail) => {
            Ok(Json(ApiResponse::success("Detail transaksi berhasil ditambahkan", new_detail)))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal menambahkan detail transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[patch("/transaksi/<id_transaksi>/detail/<id_detail>", data = "<detail>")]
pub async fn update_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32,
    id_detail: i32,
    detail: Json<DetailTransaksi>
) -> Result<Json<ApiResponse<DetailTransaksi>>, (Status, Json<ErrorResponse>)> {
    if detail.id != id_detail || detail.id_transaksi != id_transaksi {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("ID tidak sesuai", "INVALID_ID"))
        ));
    }

    match TransaksiService::get_transaksi_by_id(db.inner().clone(), id_transaksi).await {
        Ok(transaksi) => {
            if !transaksi.can_be_modified() {
                return Err((
                    Status::Forbidden,
                    Json(ErrorResponse::new(
                        "Transaksi tidak dapat dimodifikasi karena sudah selesai atau dibatalkan",
                        "TRANSACTION_IMMUTABLE"
                    ))
                ));
            }
        }
        Err(_) => {
            return Err((
                Status::NotFound,
                Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
            ));
        }
    }

    match TransaksiService::update_detail_transaksi(db.inner().clone(), &detail).await {
        Ok(updated_detail) => {
            Ok(Json(ApiResponse::success("Detail transaksi berhasil diperbarui", updated_detail)))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal memperbarui detail transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[delete("/transaksi/<id_transaksi>/detail/<id_detail>")]
pub async fn delete_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32,
    id_detail: i32
) -> Result<Json<ApiResponse<String>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::get_transaksi_by_id(db.inner().clone(), id_transaksi).await {
        Ok(transaksi) => {
            if !transaksi.can_be_modified() {
                return Err((
                    Status::Forbidden,
                    Json(ErrorResponse::new(
                        "Transaksi tidak dapat dimodifikasi karena sudah selesai atau dibatalkan",
                        "TRANSACTION_IMMUTABLE"
                    ))
                ));
            }
        }
        Err(_) => {
            return Err((
                Status::NotFound,
                Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
            ));
        }
    }

    match TransaksiService::delete_detail_transaksi(db.inner().clone(), id_detail, id_transaksi).await {
        Ok(_) => {
            Ok(Json(ApiResponse::success("Detail transaksi berhasil dihapus", "Deleted".to_string())))
        }
        Err(_) => {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse::new("Gagal menghapus detail transaksi", "INTERNAL_ERROR"))
            ))
        }
    }
}

#[autometrics]
#[get("/transaksi/<id>/full")]
pub async fn get_transaksi_with_details(
    db: &State<Pool<Any>>, 
    id: i32
) -> Result<Json<ApiResponse<crate::transaksi_penjualan::dto::transaksi_request::TransaksiWithDetailsResponse>>, (Status, Json<ErrorResponse>)> {
    let transaksi = match TransaksiService::get_transaksi_by_id(db.inner().clone(), id).await {
        Ok(t) => t,
        Err(_) => {
            return Err((
                Status::NotFound,
                Json(ErrorResponse::new("Transaksi tidak ditemukan", "NOT_FOUND"))
            ));
        }
    };

    let details = match TransaksiService::get_detail_by_transaksi_id(db.inner().clone(), id).await {
        Ok(d) => d,
        Err(_) => vec![],
    };

    let response = crate::transaksi_penjualan::dto::transaksi_request::TransaksiWithDetailsResponse {
        id: transaksi.id,
        id_pelanggan: transaksi.id_pelanggan,
        nama_pelanggan: transaksi.nama_pelanggan,
        tanggal_transaksi: transaksi.tanggal_transaksi,
        total_harga: transaksi.total_harga,
        status: transaksi.status.to_string(),
        catatan: transaksi.catatan,
        detail_transaksi: details,
    };

    Ok(Json(ApiResponse::success("Data transaksi berhasil diambil", response)))
}

#[autometrics]
#[post("/transaksi/validate-stock", data = "<products>")]
pub async fn validate_product_stock(
    products: Json<Vec<crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest>>
) -> Result<Json<ApiResponse<String>>, (Status, Json<ErrorResponse>)> {
    match TransaksiService::validate_product_stock(&products).await {
        Ok(_) => {
            Ok(Json(ApiResponse::success("Semua produk tersedia", "Valid".to_string())))
        }
        Err(err_msg) => {
            Err((
                Status::BadRequest,
                Json(ErrorResponse::new(&err_msg, "INSUFFICIENT_STOCK"))
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{routes, uri, Rocket, async_test};
    use sqlx::any::install_default_drivers;
    use crate::transaksi_penjualan::model::transaksi::Transaksi;

    async fn setup() -> Rocket<rocket::Build> {
        install_default_drivers();
        
        let db = sqlx::any::AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        
        sqlx::migrate!("migrations/test")
            .run(&db)
            .await
            .unwrap();

        rocket::build()
            .manage(db.clone())
            .mount("/", routes![
                get_all_transaksi, create_transaksi, get_transaksi_by_id, 
                update_transaksi, delete_transaksi, complete_transaksi, cancel_transaksi,
                get_detail_transaksi, add_detail_transaksi, update_detail_transaksi, delete_detail_transaksi,
                get_transaksi_with_details, validate_product_stock
            ])
    }

    #[async_test]
    async fn test_create_transaksi_with_validation() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi_request = crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Castorice".to_string(),
            catatan: Some("Test transaction".to_string()),
            detail_transaksi: vec![
                crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Contoh Produk".to_string(),
                    harga_satuan: 10000.0,
                    jumlah: 2,
                },
            ],
        };

        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi_request)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let body: ApiResponse<Transaksi> = response.into_json().await.unwrap();
        assert!(body.success);
        assert!(body.data.is_some());
        if let Some(transaksi) = body.data {
            assert_eq!(transaksi.nama_pelanggan, new_transaksi_request.nama_pelanggan);
            assert!(transaksi.total_harga > 0.0);
        }
    }

    #[async_test]
    async fn test_get_all_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let response = client.get("/transaksi").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        
        let body: ApiResponse<crate::transaksi_penjualan::service::transaksi::TransaksiSearchResult> = response.into_json().await.unwrap();
        assert!(body.success);
    }

    #[async_test]
    async fn test_validate_product_stock() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let products = vec![
            crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest {
                id_produk: 1,
                nama_produk: "Valid Product".to_string(),
                harga_satuan: 100000.0,
                jumlah: 50,
            },
        ];

        let response = client.post(uri!(super::validate_product_stock))
            .json(&products)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[async_test]
    async fn test_get_transaksi_with_details() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi_request = crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Test Full Details".to_string(),
            catatan: Some("Test transaction with details".to_string()),
            detail_transaksi: vec![
                crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Test Product".to_string(),
                    harga_satuan: 50000.0,
                    jumlah: 2,
                },
            ],
        };

        let create_response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi_request)
            .dispatch()
            .await;

        assert_eq!(create_response.status(), Status::Ok);
        let create_body: ApiResponse<Transaksi> = create_response.into_json().await.unwrap();
        let created_transaksi = create_body.data.unwrap();

        let get_response = client.get(format!("/transaksi/{}/full", created_transaksi.id)).dispatch().await;
        assert_eq!(get_response.status(), Status::Ok);

        let get_body: ApiResponse<crate::transaksi_penjualan::dto::transaksi_request::TransaksiWithDetailsResponse> = get_response.into_json().await.unwrap();
        assert!(get_body.success);
        
        if let Some(full_transaksi) = get_body.data {
            assert_eq!(full_transaksi.id, created_transaksi.id);
            assert_eq!(full_transaksi.nama_pelanggan, "Test Full Details");
            assert!(!full_transaksi.detail_transaksi.is_empty());
        }
    }

    #[async_test]
    async fn test_transaksi_state_transitions() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi_request = crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "State Test".to_string(),
            catatan: None,
            detail_transaksi: vec![
                crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "State Test Product".to_string(),
                    harga_satuan: 100000.0,
                    jumlah: 1,
                },
            ],
        };

        let create_response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi_request)
            .dispatch()
            .await;

        let create_body: ApiResponse<Transaksi> = create_response.into_json().await.unwrap();
        let created_transaksi = create_body.data.unwrap();

        let complete_response = client.put(format!("/transaksi/{}/complete", created_transaksi.id)).dispatch().await;
        assert_eq!(complete_response.status(), Status::Ok);

        let complete_body: ApiResponse<Transaksi> = complete_response.into_json().await.unwrap();
        if let Some(completed_transaksi) = complete_body.data {
            assert_eq!(completed_transaksi.status.to_string(), "SELESAI");
        }

        let mut update_transaksi = created_transaksi.clone();
        update_transaksi.nama_pelanggan = "Updated Name".to_string();

        let update_response = client.patch(format!("/transaksi/{}", created_transaksi.id))
            .json(&update_transaksi)
            .dispatch()
            .await;

        assert_eq!(update_response.status(), Status::Forbidden);
    }

    #[async_test]
    async fn test_detail_transaksi_crud() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi_request = crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Detail CRUD Test".to_string(),
            catatan: None,
            detail_transaksi: vec![
                crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Initial Product".to_string(),
                    harga_satuan: 50000.0,
                    jumlah: 1,
                },
            ],
        };

        let create_response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi_request)
            .dispatch()
            .await;

        let create_body: ApiResponse<Transaksi> = create_response.into_json().await.unwrap();
        let created_transaksi = create_body.data.unwrap();

        let get_details_response = client.get(format!("/transaksi/{}/detail", created_transaksi.id)).dispatch().await;
        assert_eq!(get_details_response.status(), Status::Ok);

        let get_details_body: ApiResponse<Vec<DetailTransaksi>> = get_details_response.into_json().await.unwrap();
        assert!(get_details_body.success);
        
        if let Some(details) = get_details_body.data {
            assert_eq!(details.len(), 1);
            
            let mut detail_to_update = details[0].clone();
            detail_to_update.update_jumlah(3);

            let update_detail_response = client.patch(format!("/transaksi/{}/detail/{}", created_transaksi.id, detail_to_update.id))
                .json(&detail_to_update)
                .dispatch()
                .await;

            assert_eq!(update_detail_response.status(), Status::Ok);

            let delete_detail_response = client.delete(format!("/transaksi/{}/detail/{}", created_transaksi.id, detail_to_update.id)).dispatch().await;
            assert_eq!(delete_detail_response.status(), Status::Ok);
        }
    }

    #[async_test]
    async fn test_error_handling() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let response = client.get("/transaksi/99999").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);

        let invalid_request = crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "".to_string(), 
            catatan: None,
            detail_transaksi: vec![],       
        };

        let response = client.post(uri!(super::create_transaksi))
            .json(&invalid_request)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::BadRequest);
    }
}