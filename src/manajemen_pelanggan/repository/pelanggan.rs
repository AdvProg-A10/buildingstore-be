use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::NaiveDate;

use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

pub struct PelangganRepository;

impl PelangganRepository {
    pub async fn create_pelanggan(mut db: PoolConnection<Any>, pelanggan: &Pelanggan) -> Result<Pelanggan, sqlx::Error> {
        let result = sqlx::query("
                INSERT INTO pelanggan (nama, alamat, no_telp, tanggal_gabung)
                VALUES ($1, $2, $3, $4)
                RETURNING id, nama, alamat, no_telp, tanggal_gabung
            ")
            .bind(&pelanggan.nama)
            .bind(&pelanggan.alamat)
            .bind(&pelanggan.no_telp)
            .bind(pelanggan.tanggal_gabung.to_string())
            .fetch_one(&mut *db)
            .await?;
        
        let pelanggan = Self::parse_row_to_pelanggan(result);

        Ok(pelanggan)
    }

    pub async fn get_pelanggan_by_id(mut db: PoolConnection<Any>, id: i32) -> Result<Pelanggan, sqlx::Error> {
        let result = sqlx::query("
                SELECT id, nama, alamat, no_telp, tanggal_gabung
                FROM pelanggan
                WHERE id = $1
            ")
            .bind(id)
            .fetch_one(&mut *db)
            .await?;
        
        let pelanggan = Self::parse_row_to_pelanggan(result);

        Ok(pelanggan)
    }

    pub async fn update_pelanggan(mut db: PoolConnection<Any>, pelanggan: &Pelanggan) -> Result<Pelanggan, sqlx::Error> {
        let result = sqlx::query("
        UPDATE pelanggan
        SET nama = $1, alamat = $2, no_telp = $3, tanggal_gabung = $4
        WHERE id = $5
        RETURNING id, nama, alamat, no_telp, tanggal_gabung
        ")
            .bind(&pelanggan.nama)
            .bind(&pelanggan.alamat)
            .bind(&pelanggan.no_telp)
            .bind(pelanggan.tanggal_gabung.to_string())
            .bind(pelanggan.id)
            .fetch_one(&mut *db)
            .await?;
        
        let pelanggan = Self::parse_row_to_pelanggan(result);

        Ok(pelanggan)
    }
    
    pub async fn delete_pelanggan(mut db: PoolConnection<Any>, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("
        DELETE FROM pelanggan
                WHERE id = $1
                ")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_all_pelanggan(mut db: PoolConnection<Any>) -> Result<Vec<Pelanggan>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, nama, alamat, no_telp, tanggal_gabung
                FROM pelanggan
            ")
            .fetch_all(&mut *db)
            .await?;
        
        let mut pelanggan_list = Vec::new();
        for row in rows {
            let pelanggan = Self::parse_row_to_pelanggan(row);
            pelanggan_list.push(pelanggan);
        }
        
        Ok(pelanggan_list)
    }

    fn parse_row_to_pelanggan(row: AnyRow) -> Pelanggan {
        Pelanggan {
            id: row.get("id"),
            nama: row.get("nama"),
            alamat: row.get("alamat"),
            no_telp: row.get("no_telp"),
            tanggal_gabung: NaiveDate::parse_from_str(&row.get::<String, _>("tanggal_gabung"), "%Y-%m-%d").unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::any::install_default_drivers;
    use sqlx::{Any, Pool};
    use sqlx::any::AnyPoolOptions;
    use rocket::async_test;
    use chrono::Utc;

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

        let pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "1234567890".to_string());
        let created_pelanggan = PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan).await.unwrap();

        assert_eq!(created_pelanggan.nama, "Castorice");
        assert_eq!(created_pelanggan.alamat, "Styxia");
        assert_eq!(created_pelanggan.no_telp, "1234567890");
    }

    #[async_test]
    async fn test_get_pelanggan_by_id() {
        let db = setup().await;

        let pelanggan = Pelanggan::new("Tribbie".to_string(), "Okhema".to_string(), "0987654321".to_string());
        let created_pelanggan = PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan).await.unwrap();

        let fetched_pelanggan = PelangganRepository::get_pelanggan_by_id(db.acquire().await.unwrap(), created_pelanggan.id).await.unwrap();

        assert_eq!(fetched_pelanggan.nama, "Tribbie");
        assert_eq!(fetched_pelanggan.alamat, "Okhema");
        assert_eq!(fetched_pelanggan.no_telp, "0987654321");
        assert_eq!(fetched_pelanggan.tanggal_gabung, Utc::now().date_naive());
    }

    #[async_test]
    async fn test_get_pelanggan_by_id_not_found() {
        let db = setup().await;

        let result = PelangganRepository::get_pelanggan_by_id(db.acquire().await.unwrap(), 999).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_update_pelanggan() {
        let db = setup().await;

        let pelanggan = Pelanggan::new("Hyacine".to_string(), "The Grove".to_string(), "1122334455".to_string());
        let created_pelanggan = PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan).await.unwrap();

        let updated_pelanggan = Pelanggan {
            id: created_pelanggan.id,
            nama: "Hyacine".to_string(),
            alamat: "Okhema".to_string(),
            no_telp: "1234567890".to_string(),
            tanggal_gabung: created_pelanggan.tanggal_gabung,
        };

        let result = PelangganRepository::update_pelanggan(db.acquire().await.unwrap(), &updated_pelanggan).await.unwrap();
        assert_eq!(result.nama, "Hyacine");
        assert_eq!(result.no_telp, "1234567890");
    }

    #[async_test]
    async fn test_update_pelanggan_not_found() {
        let db = setup().await;

        let pelanggan = Pelanggan::new("Anaxa".to_string(), "The Grove".to_string(), "5566778899".to_string());
        let created_pelanggan = PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan).await.unwrap();

        let updated_pelanggan = Pelanggan {
            id: 999,
            nama: "Anaxagoras".to_string(),
            alamat: "Dragonbone City".to_string(),
            no_telp: "5566778899".to_string(),
            tanggal_gabung: created_pelanggan.tanggal_gabung,
        };

        let result = PelangganRepository::update_pelanggan(db.acquire().await.unwrap(), &updated_pelanggan).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_delete_pelanggan() {
        let db = setup().await;

        let pelanggan = Pelanggan::new("Aglaea".to_string(), "Okhema".to_string(), "9988776655".to_string());
        let created_pelanggan = PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan).await.unwrap();

        PelangganRepository::delete_pelanggan(db.acquire().await.unwrap(), created_pelanggan.id).await.unwrap();

        let result = PelangganRepository::get_pelanggan_by_id(db.acquire().await.unwrap(), created_pelanggan.id).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_get_all_pelanggan() {
        let db = setup().await;

        let pelanggan1 = Pelanggan::new("Polyxia".to_string(), "River of Souls".to_string(), "2233445566".to_string());
        let pelanggan2 = Pelanggan::new("Pollux".to_string(), "River of Souls".to_string(), "3344556677".to_string());

        PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan1).await.unwrap();
        PelangganRepository::create_pelanggan(db.acquire().await.unwrap(), &pelanggan2).await.unwrap();

        let result = PelangganRepository::get_all_pelanggan(db.acquire().await.unwrap()).await.unwrap();
        assert_eq!(result.len(), 2);
    }
}