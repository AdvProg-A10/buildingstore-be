use sqlx::{Any, Pool};
use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::repository::transaksi::TransaksiRepository;
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

pub struct TransaksiService;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TransaksiSearchParams {
    pub sort: Option<String>,
    pub filter: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub id_pelanggan: Option<i32>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TransaksiSearchResult {
    pub data: Vec<Transaksi>,
    pub total_count: usize,
    pub page: usize,
    pub limit: usize,
    pub total_pages: usize,
}

impl TransaksiSearchResult {
    pub fn empty() -> Self {
        Self {
            data: vec![],
            total_count: 0,
            page: 1,
            limit: 10,
            total_pages: 0,
        }
    }
}

impl TransaksiService {
    pub async fn create_transaksi(db: Pool<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::create_transaksi(db_connection, transaksi).await
    }

    pub async fn create_transaksi_with_details(
        db: Pool<Any>, 
        request: &crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest
    ) -> Result<Transaksi, sqlx::Error> {
        if let Err(_err_msg) = request.validate() {
            return Err(sqlx::Error::RowNotFound);
        }

        if let Err(_err_msg) = Self::validate_product_stock(&request.detail_transaksi).await {
            return Err(sqlx::Error::RowNotFound);
        }

        let product_prices = Self::fetch_product_prices(&request.detail_transaksi).await?;
        let total_harga = request.calculate_total(&product_prices);

        let transaksi = Transaksi::new(
            request.id_pelanggan,
            request.nama_pelanggan.clone(),
            total_harga,
            request.catatan.clone(),
        );

        let db_connection = db.acquire().await?;
        let created_transaksi = TransaksiRepository::create_transaksi(db_connection, &transaksi).await?;

        for detail_request in &request.detail_transaksi {
            let harga_satuan = product_prices.get(&detail_request.id_produk).unwrap_or(&0.0);
            let detail = detail_request.to_detail_transaksi(created_transaksi.id, *harga_satuan);
            
            let db_connection = db.acquire().await?;
            TransaksiRepository::create_detail_transaksi(db_connection, &detail).await?;
            
            Self::reduce_product_stock(detail_request.id_produk, detail_request.jumlah).await?;
        }

        Ok(created_transaksi)
    }

    async fn reduce_product_stock(product_id: i32, quantity: u32) -> Result<(), sqlx::Error> {
        println!("Mengurangi stok produk ID {} sebanyak {}", product_id, quantity);
        Ok(())
    }

    async fn restore_product_stock(product_id: i32, quantity: u32) -> Result<(), sqlx::Error> {
        println!("Mengembalikan stok produk ID {} sebanyak {}", product_id, quantity);
        Ok(())
    }

    pub async fn validate_product_stock(
        detail_requests: &[crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest]
    ) -> Result<(), String> {
        for detail in detail_requests {
            let available_stock = Self::get_product_stock(detail.id_produk).await;

            if available_stock == 0 {
                return Err(format!("Produk dengan ID {} tidak ditemukan atau stok habis", detail.id_produk));
            }

            if detail.jumlah > available_stock {
                return Err(format!(
                    "Stok produk '{}' tidak mencukupi. Tersedia: {}, Diminta: {}", 
                    detail.nama_produk, available_stock, detail.jumlah
                ));
            }
        }

        Ok(())
    }

    async fn get_product_stock(product_id: i32) -> u32 {
        match product_id {
            1 => 100,
            2 => 50,
            3 => 25,
            4 => 15,
            5 => 30,
            _ => 0,
        }
    }

    async fn fetch_product_prices(
        detail_requests: &[crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest]
    ) -> Result<std::collections::HashMap<i32, f64>, sqlx::Error> {
        let mut prices = std::collections::HashMap::new();
        
        for detail in detail_requests {
            let price = match detail.id_produk {
                1 => 100000.0,  
                2 => 250000.0,   
                3 => 500000.0, 
                4 => 75000.0,  
                5 => 150000.0,  
                _ => 50000.0,  
            };
            prices.insert(detail.id_produk, price);
        }

        Ok(prices)
    }

    pub async fn get_transaksi_by_id(db: Pool<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_transaksi_by_id(db_connection, id).await
    }

    pub async fn update_transaksi(db: Pool<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let existing_transaksi = Self::get_transaksi_by_id(db.clone(), transaksi.id).await?;
        
        if !existing_transaksi.can_be_modified() {
            return Err(sqlx::Error::RowNotFound);
        }

        let db_connection = db.acquire().await?;
        TransaksiRepository::update_transaksi(db_connection, transaksi).await
    }

    pub async fn delete_transaksi(db: Pool<Any>, id: i32) -> Result<(), sqlx::Error> {
        let existing_transaksi = Self::get_transaksi_by_id(db.clone(), id).await?;
        
        if !existing_transaksi.can_be_cancelled() {
            return Err(sqlx::Error::RowNotFound); 
        }

        let details = Self::get_detail_by_transaksi_id(db.clone(), id).await?;
        for detail in details {
            Self::restore_product_stock(detail.id_produk, detail.jumlah).await?;
        }

        let db_connection_detail = db.acquire().await?;
        TransaksiRepository::delete_detail_by_transaksi_id(db_connection_detail, id).await?;

        let db_connection = db.acquire().await?;
        TransaksiRepository::delete_transaksi(db_connection, id).await
    }

    pub async fn get_all_transaksi(db: Pool<Any>) -> Result<Vec<Transaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_all_transaksi(db_connection).await
    }

    pub async fn get_transaksi_by_pelanggan(db: Pool<Any>, id_pelanggan: i32) -> Result<Vec<Transaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_transaksi_by_pelanggan(db_connection, id_pelanggan).await
    }

    pub async fn get_transaksi_by_status(db: Pool<Any>, status: &StatusTransaksi) -> Result<Vec<Transaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_transaksi_by_status(db_connection, status).await
    }

    pub async fn complete_transaksi(db: Pool<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let mut transaksi = Self::get_transaksi_by_id(db.clone(), id).await?;
        
        if !transaksi.can_be_modified() {
            return Err(sqlx::Error::RowNotFound);
        }

        transaksi.update_status(StatusTransaksi::Selesai);
        Self::update_transaksi(db, &transaksi).await
    }

    pub async fn cancel_transaksi(db: Pool<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let mut transaksi = Self::get_transaksi_by_id(db.clone(), id).await?;
        
        if !transaksi.status.can_be_cancelled() {
            return Err(sqlx::Error::RowNotFound);
        }

        let details = Self::get_detail_by_transaksi_id(db.clone(), id).await?;
        for detail in details {
            Self::restore_product_stock(detail.id_produk, detail.jumlah).await?;
        }

        transaksi.update_status(StatusTransaksi::Dibatalkan);
        Self::update_transaksi(db, &transaksi).await
    }

    pub async fn add_detail_transaksi(db: Pool<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db.clone(), detail.id_transaksi).await?;
        
        if !transaksi.can_be_modified() {
            return Err(sqlx::Error::RowNotFound);
        }

        let db_connection = db.acquire().await?;
        let created_detail = TransaksiRepository::create_detail_transaksi(db_connection, detail).await?;

        Self::recalculate_transaction_total(db, detail.id_transaksi).await?;

        Ok(created_detail)
    }

    pub async fn get_detail_by_transaksi_id(db: Pool<Any>, id_transaksi: i32) -> Result<Vec<DetailTransaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_detail_by_transaksi_id(db_connection, id_transaksi).await
    }

    pub async fn update_detail_transaksi(db: Pool<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db.clone(), detail.id_transaksi).await?;
        
        if !transaksi.can_be_modified() {
            return Err(sqlx::Error::RowNotFound);
        }

        let db_connection = db.acquire().await?;
        let updated_detail = TransaksiRepository::update_detail_transaksi(db_connection, detail).await?;

        Self::recalculate_transaction_total(db, detail.id_transaksi).await?;

        Ok(updated_detail)
    }

    pub async fn delete_detail_transaksi(db: Pool<Any>, id: i32, id_transaksi: i32) -> Result<(), sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db.clone(), id_transaksi).await?;
        
        if !transaksi.can_be_modified() {
            return Err(sqlx::Error::RowNotFound);
        }

        let details = Self::get_detail_by_transaksi_id(db.clone(), id_transaksi).await?;
        if let Some(detail_to_delete) = details.iter().find(|d| d.id == id) {
            Self::restore_product_stock(detail_to_delete.id_produk, detail_to_delete.jumlah).await?;
        }

        let db_connection = db.acquire().await?;
        TransaksiRepository::delete_detail_transaksi(db_connection, id).await?;

        Self::recalculate_transaction_total(db, id_transaksi).await?;

        Ok(())
    }

    async fn recalculate_transaction_total(db: Pool<Any>, id_transaksi: i32) -> Result<(), sqlx::Error> {
        let details = Self::get_detail_by_transaksi_id(db.clone(), id_transaksi).await?;
        let total: f64 = details.iter().map(|d| d.subtotal).sum();

        let mut transaksi = Self::get_transaksi_by_id(db.clone(), id_transaksi).await?;
        transaksi.update_total_harga(total);

        let db_connection = db.acquire().await?;
        TransaksiRepository::update_transaksi(db_connection, &transaksi).await?;

        Ok(())
    }

    pub async fn search_transaksi_with_pagination(
        db: Pool<Any>,
        search_params: &TransaksiSearchParams
    ) -> Result<TransaksiSearchResult, sqlx::Error> {
        let mut transaksi_list = if let Some(customer_id) = search_params.id_pelanggan {
            Self::get_transaksi_by_pelanggan(db.clone(), customer_id).await?
        } else if let Some(ref status_str) = search_params.status {
            if let Some(status_enum) = StatusTransaksi::from_string(status_str) {
                Self::get_transaksi_by_status(db.clone(), &status_enum).await?
            } else {
                return Ok(TransaksiSearchResult::empty());
            }
        } else {
            Self::get_all_transaksi(db).await?
        };

        if let Some(ref sort_strategy) = search_params.sort {
            transaksi_list = Self::sort_transaksi(transaksi_list, sort_strategy);
        }

        if let Some(ref filter_strategy) = search_params.filter {
            if let Some(ref keyword_value) = search_params.keyword {
                transaksi_list = Self::filter_transaksi(transaksi_list, filter_strategy, keyword_value);
            }
        }

        let total_count = transaksi_list.len();
        let page = search_params.page.unwrap_or(1);
        let limit = search_params.limit.unwrap_or(10);
        let offset = (page - 1) * limit;

        let paginated_list = transaksi_list
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(TransaksiSearchResult {
            data: paginated_list,
            total_count,
            page,
            limit,
            total_pages: (total_count + limit - 1) / limit,
        })
    }

    pub fn sort_transaksi(mut transaksi_list: Vec<Transaksi>, sort_by: &str) -> Vec<Transaksi> {
        match sort_by.to_lowercase().as_str() {
            "tanggal" | "tanggal_transaksi" => {
                transaksi_list.sort_by(|a, b| a.tanggal_transaksi.cmp(&b.tanggal_transaksi));
            }
            "tanggal_desc" => {
                transaksi_list.sort_by(|a, b| b.tanggal_transaksi.cmp(&a.tanggal_transaksi));
            }
            "total" | "total_harga" => {
                transaksi_list.sort_by(|a, b| a.total_harga.partial_cmp(&b.total_harga).unwrap());
            }
            "total_desc" => {
                transaksi_list.sort_by(|a, b| b.total_harga.partial_cmp(&a.total_harga).unwrap());
            }
            "pelanggan" | "nama_pelanggan" => {
                transaksi_list.sort_by(|a, b| a.nama_pelanggan.cmp(&b.nama_pelanggan));
            }
            "status" => {
                transaksi_list.sort_by(|a, b| a.status.to_string().cmp(&b.status.to_string()));
            }
            _ => {
                transaksi_list.sort_by(|a, b| b.tanggal_transaksi.cmp(&a.tanggal_transaksi));
            }
        }
        transaksi_list
    }

    pub fn filter_transaksi(transaksi_list: Vec<Transaksi>, filter_by: &str, keyword: &str) -> Vec<Transaksi> {
        let keyword = keyword.to_lowercase();
        
        transaksi_list.into_iter().filter(|transaksi| {
            match filter_by.to_lowercase().as_str() {
                "id" => transaksi.id.to_string().contains(&keyword),
                "nama_pelanggan" | "pelanggan" => transaksi.nama_pelanggan.to_lowercase().contains(&keyword),
                "status" => transaksi.status.to_string().to_lowercase().contains(&keyword),
                "total" | "total_harga" => transaksi.total_harga.to_string().contains(&keyword),
                "catatan" => {
                    if let Some(ref catatan) = transaksi.catatan {
                        catatan.to_lowercase().contains(&keyword)
                    } else {
                        false
                    }
                }
                _ => {
                    transaksi.nama_pelanggan.to_lowercase().contains(&keyword) ||
                    transaksi.status.to_string().to_lowercase().contains(&keyword) ||
                    (transaksi.catatan.as_ref().map_or(false, |c| c.to_lowercase().contains(&keyword)))
                }
            }
        }).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::any::install_default_drivers;
    use sqlx::{Any, Pool};
    use rocket::async_test;

    async fn setup() -> Pool<Any> {
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
        
        db
    }

    #[async_test]
    async fn test_create_transaksi() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            1,
            "Castorice".to_string(),
            150000.0,
            Some("Test transaction".to_string()),
        );
        let created_transaksi = TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();

        assert_eq!(created_transaksi.id_pelanggan, 1);
        assert_eq!(created_transaksi.nama_pelanggan, "Castorice");
        assert_eq!(created_transaksi.total_harga, 150000.0);
        assert_eq!(created_transaksi.status, StatusTransaksi::MasihDiproses);
    }

    #[async_test]
    async fn test_get_transaksi_by_id() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            2,
            "Tribbie".to_string(),
            200000.0,
            None,
        );
        let created_transaksi = TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();

        let fetched_transaksi = TransaksiService::get_transaksi_by_id(db.clone(), created_transaksi.id).await.unwrap();

        assert_eq!(fetched_transaksi.id_pelanggan, 2);
        assert_eq!(fetched_transaksi.nama_pelanggan, "Tribbie");
        assert_eq!(fetched_transaksi.total_harga, 200000.0);
    }

    #[async_test]
    async fn test_create_detail_transaksi() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            1,
            "Hyacine".to_string(),
            500000.0,
            None,
        );
        let created_transaksi = TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();

        let detail = DetailTransaksi::new(
            created_transaksi.id,
            101,
            15000000.0,
            1,
        );
        let created_detail = TransaksiService::add_detail_transaksi(db.clone(), &detail).await.unwrap();

        assert_eq!(created_detail.id_transaksi, created_transaksi.id);
        assert_eq!(created_detail.id_produk, 101);
        assert_eq!(created_detail.subtotal, 15000000.0);
    }

    #[async_test]
    async fn test_get_all_transaksi() {
        let db = setup().await;

        let transaksi1 = Transaksi::new(1, "Alice".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(2, "Bob".to_string(), 200000.0, None);

        TransaksiService::create_transaksi(db.clone(), &transaksi1).await.unwrap();
        TransaksiService::create_transaksi(db.clone(), &transaksi2).await.unwrap();

        let all_transaksi = TransaksiService::get_all_transaksi(db.clone()).await.unwrap();
        
        assert_eq!(all_transaksi.len(), 2);
        assert!(all_transaksi.iter().any(|t| t.nama_pelanggan == "Alice"));
        assert!(all_transaksi.iter().any(|t| t.nama_pelanggan == "Bob"));
    }

    #[async_test]
    async fn test_simple_data_types() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            99,
            "Data Type Test".to_string(),
            999.99,
            Some("Testing simple data types".to_string()),
        );

        let created = TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();
        
        assert!(created.id > 0);
        assert_eq!(created.id_pelanggan, 99);
        assert_eq!(created.total_harga, 999.99);
        assert!(!created.tanggal_transaksi.is_empty());
        
        println!("Created transaksi with simple data types: {:?}", created);
    }
}