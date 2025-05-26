use sqlx::{Any, Pool};
use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;
use crate::manajemen_pelanggan::repository::pelanggan::PelangganRepository;
use crate::manajemen_pelanggan::service::{sort_context::SortContext, sort::SortByNama, sort::SortByTanggalGabung,
    filter_context::FilterContext, filter::FilterByNama, filter::FilterByTanggalGabungPrev, filter::FilterByTanggalGabungAfter};

pub struct PelangganService;

impl PelangganService {
    pub async fn create_pelanggan(db: Pool<Any>, pelanggan: &Pelanggan) -> Result<Pelanggan, sqlx::Error> {
        let conn = db.acquire().await?;
        PelangganRepository::create_pelanggan(conn, pelanggan).await
    }

    pub async fn get_pelanggan_by_id(db: Pool<Any>, id: i32) -> Result<Pelanggan, sqlx::Error> {
        let conn = db.acquire().await?;
        PelangganRepository::get_pelanggan_by_id(conn, id).await
    }

    pub async fn get_all_pelanggan(db: Pool<Any>) -> Result<Vec<Pelanggan>, sqlx::Error> {
        let conn = db.acquire().await?;
        PelangganRepository::get_all_pelanggan(conn).await
    }

    pub async fn update_pelanggan(db: Pool<Any>, pelanggan: &Pelanggan) -> Result<Pelanggan, sqlx::Error> {
        let conn = db.acquire().await?;
        PelangganRepository::update_pelanggan(conn, pelanggan).await
    }

    pub async fn delete_pelanggan(db: Pool<Any>, id: i32) -> Result<(), sqlx::Error> {
        let conn = db.acquire().await?;
        PelangganRepository::delete_pelanggan(conn, id).await
    }

    pub fn sort_pelanggan(pelanggan: Vec<Pelanggan>, sort_strategy: &str) -> Vec<Pelanggan> {
        let mut pelanggan = pelanggan;
        let mut sort_context = SortContext::new();
        match sort_strategy {
            "nama" => sort_context.set_strategy(Box::new(SortByNama)),
            "tanggal_gabung" => sort_context.set_strategy(Box::new(SortByTanggalGabung)),
            _ => {} // No sorting if the strategy is not recognized
        }
        sort_context.execute_sort(&mut pelanggan);
        pelanggan
    }

    pub fn filter_pelanggan(pelanggan: Vec<Pelanggan>, filter_strategy: &str, keyword: &str) -> Vec<Pelanggan> {
        let mut pelanggan = pelanggan;
        let mut filter_context = FilterContext::new();
        match filter_strategy {
            "nama" => filter_context.set_strategy(Box::new(FilterByNama)),
            "tanggal_gabung_prev" => filter_context.set_strategy(Box::new(FilterByTanggalGabungPrev)),
            "tanggal_gabung_after" => filter_context.set_strategy(Box::new(FilterByTanggalGabungAfter)),
            _ => {} // No filtering if the strategy is not recognized
        }
        filter_context.execute_filter(&mut pelanggan, keyword);
        pelanggan
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::{Any, Pool};
    use sqlx::any::{AnyPoolOptions, install_default_drivers};
    use chrono::NaiveDate;
    use rocket::async_test;

    async fn setup() -> Pool<Any> {
        install_default_drivers();
        let db = AnyPoolOptions::new()
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
    async fn test_create_pelanggan() {
        let db = setup().await;
        let pelanggan = Pelanggan {
            id: 0,
            nama: "John Doe".to_string(),
            alamat: "123 Main St".to_string(),
            no_telp: "08123456789".to_string(),
            tanggal_gabung: NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
        };

        let result = PelangganService::create_pelanggan(db.clone(), &pelanggan).await;
        assert!(result.is_ok());
        let created_pelanggan = result.unwrap();
        assert_eq!(created_pelanggan.nama, pelanggan.nama);
        assert_eq!(created_pelanggan.alamat, pelanggan.alamat);
        assert_eq!(created_pelanggan.no_telp, pelanggan.no_telp);
    }

    #[async_test]
    async fn test_get_pelanggan_by_id() {
        let db = setup().await;
        let pelanggan = Pelanggan {
            id: 0,
            nama: "Jane Doe".to_string(),
            alamat: "456 Elm St".to_string(),
            no_telp: "08123456789".to_string(),
            tanggal_gabung: NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
        };

        let created_pelanggan = PelangganService::create_pelanggan(db.clone(), &pelanggan).await.unwrap();
        let result = PelangganService::get_pelanggan_by_id(db.clone(), created_pelanggan.id).await;
        assert!(result.is_ok());
        let fetched_pelanggan = result.unwrap();
        assert_eq!(fetched_pelanggan.nama, pelanggan.nama);
        assert_eq!(fetched_pelanggan.alamat, pelanggan.alamat);
        assert_eq!(fetched_pelanggan.no_telp, pelanggan.no_telp);
    }

    #[async_test]
    async fn test_get_invalid_pelanggan_by_id() {
        let db = setup().await;
        let result = PelangganService::get_pelanggan_by_id(db.clone(), 999).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_get_all_pelanggan() {
        let db = setup().await;
        let pelanggan1 = Pelanggan {
            id: 0,
            nama: "Alice".to_string(),
            alamat: "789 Oak St".to_string(),
            no_telp: "08123456789".to_string(),
            tanggal_gabung: NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
        };
        let pelanggan2 = Pelanggan {
            id: 0,
            nama: "Bob".to_string(),
            alamat: "101 Pine St".to_string(),
            no_telp: "08123456789".to_string(),
            tanggal_gabung: NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
        };

        PelangganService::create_pelanggan(db.clone(), &pelanggan1).await.unwrap();
        PelangganService::create_pelanggan(db.clone(), &pelanggan2).await.unwrap();

        let result = PelangganService::get_all_pelanggan(db.clone()).await;
        assert!(result.is_ok());
        let all_pelanggan = result.unwrap();
        assert_eq!(all_pelanggan.len(), 2);
    }

    #[async_test]
    async fn test_update_pelanggan() {
        let db = setup().await;
        let pelanggan = Pelanggan::new("Bob".to_string(), "101 Pine St".to_string(), "08123456789".to_string());

        let created_pelanggan = PelangganService::create_pelanggan(db.clone(), &pelanggan).await.unwrap();
        let updated_pelanggan = Pelanggan {
            id: created_pelanggan.id,
            nama: "Charlie Brown".to_string(),
            alamat: "111 Maple St".to_string(),
            no_telp: "08123456789".to_string(),
            tanggal_gabung: NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
        };

        let result = PelangganService::update_pelanggan(db.clone(), &updated_pelanggan).await;
        assert!(result.is_ok());
        let fetched_pelanggan = result.unwrap();
        assert_eq!(fetched_pelanggan.nama, updated_pelanggan.nama);
        assert_eq!(fetched_pelanggan.alamat, updated_pelanggan.alamat);
    }

    #[async_test]
    async fn test_update_invalid_pelanggan() {
        let db = setup().await;
        let pelanggan = Pelanggan::new("Eve".to_string(), "1212 Birch St".to_string(), "08123456789".to_string());

        PelangganService::create_pelanggan(db.clone(), &pelanggan).await.unwrap();
        let updated_pelanggan = Pelanggan {
            id: 999, // Invalid ID
            nama: "Frank".to_string(),
            alamat: "1313 Cedar St".to_string(),
            no_telp: "08123456789".to_string(),
            tanggal_gabung: NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
        };

        let result = PelangganService::update_pelanggan(db.clone(), &updated_pelanggan).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_delete_pelanggan() {
        let db = setup().await;
        let pelanggan = Pelanggan::new("Dave".to_string(), "1212 Birch St".to_string(), "08123456789".to_string());

        let created_pelanggan = PelangganService::create_pelanggan(db.clone(), &pelanggan).await.unwrap();
        let result = PelangganService::delete_pelanggan(db.clone(), created_pelanggan.id).await;
        assert!(result.is_ok());

        let result = PelangganService::get_pelanggan_by_id(db.clone(), created_pelanggan.id).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_pelanggan() {
        let pelanggan1 = Pelanggan::new("Charlie".to_string(), "111 Maple St".to_string(), "08123456789".to_string());
        let pelanggan2 = Pelanggan::new("Alice".to_string(), "789 Oak St".to_string(), "08123456789".to_string());
        let pelanggan3 = Pelanggan::new("Bob".to_string(), "101 Pine St".to_string(), "08123456789".to_string());

        let pelanggan_list = vec![pelanggan1, pelanggan2, pelanggan3];
        let sorted_pelanggan = PelangganService::sort_pelanggan(pelanggan_list.clone(), "nama");
        assert_eq!(sorted_pelanggan[0].nama, "Alice");
        assert_eq!(sorted_pelanggan[1].nama, "Bob");
        assert_eq!(sorted_pelanggan[2].nama, "Charlie");
    }

    #[test]
    fn test_filter_pelanggan() {
        let pelanggan1 = Pelanggan::new("Charlie".to_string(), "111 Maple St".to_string(), "08123456789".to_string());
        let pelanggan2 = Pelanggan::new("Alice".to_string(), "789 Oak St".to_string(), "08123456789".to_string());
        let pelanggan3 = Pelanggan::new("Bob".to_string(), "101 Pine St".to_string(), "08123456789".to_string());

        let pelanggan_list = vec![pelanggan1, pelanggan2, pelanggan3];
        let filtered_pelanggan = PelangganService::filter_pelanggan(pelanggan_list.clone(), "nama", "Alice");
        assert_eq!(filtered_pelanggan.len(), 1);
        assert_eq!(filtered_pelanggan[0].nama, "Alice");
    }
}