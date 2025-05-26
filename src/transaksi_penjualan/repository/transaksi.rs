use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::Utc;

use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

pub struct TransaksiRepository;

impl TransaksiRepository {
    pub async fn create_transaksi(mut db: PoolConnection<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let result = sqlx::query("
                INSERT INTO transaksi (id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga::text as total_harga_str, status, catatan
            ")
            .bind(transaksi.id_pelanggan)
            .bind(&transaksi.nama_pelanggan)
            .bind(&transaksi.tanggal_transaksi)
            .bind(transaksi.total_harga)
            .bind(transaksi.status.to_string())
            .bind(transaksi.catatan.as_deref().unwrap_or(""))
            .bind(&now)
            .bind(&now)
            .fetch_one(&mut *db)
            .await?;
        
        let transaksi = Self::parse_row_to_transaksi(result)?;
        Ok(transaksi)
    }

    pub async fn get_transaksi_by_id(mut db: PoolConnection<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let result = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga::text as total_harga_str, status, catatan
                FROM transaksi
                WHERE id = $1
            ")
            .bind(id)
            .fetch_one(&mut *db)
            .await?;
        
        let transaksi = Self::parse_row_to_transaksi(result)?;
        Ok(transaksi)
    }

    pub async fn update_transaksi(mut db: PoolConnection<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let result = sqlx::query("
                UPDATE transaksi
                SET id_pelanggan = $1, nama_pelanggan = $2, tanggal_transaksi = $3, 
                    total_harga = $4, status = $5, catatan = $6, updated_at = $7
                WHERE id = $8
                RETURNING id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga::text as total_harga_str, status, catatan
            ")
            .bind(transaksi.id_pelanggan)
            .bind(&transaksi.nama_pelanggan)
            .bind(&transaksi.tanggal_transaksi)
            .bind(transaksi.total_harga)
            .bind(transaksi.status.to_string())
            .bind(transaksi.catatan.as_deref().unwrap_or(""))
            .bind(&now)
            .bind(transaksi.id)
            .fetch_one(&mut *db)
            .await?;
        
        let transaksi = Self::parse_row_to_transaksi(result)?;
        Ok(transaksi)
    }

    pub async fn delete_transaksi(mut db: PoolConnection<Any>, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM transaksi WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }

    pub async fn get_all_transaksi(mut db: PoolConnection<Any>) -> Result<Vec<Transaksi>, sqlx::Error> {
        println!("🔍 Repository: Executing get_all_transaksi query with NUMERIC -> TEXT conversion");
        
        let rows = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, 
                       total_harga::text as total_harga_str, status, catatan
                FROM transaksi
                ORDER BY tanggal_transaksi DESC
            ")
            .fetch_all(&mut *db)
            .await?;
        
        println!("📊 Repository: Found {} rows", rows.len());
        
        let mut transaksi_list = Vec::new();
        for (i, row) in rows.into_iter().enumerate() {
            match Self::parse_row_to_transaksi(row) {
                Ok(transaksi) => {
                    println!("✅ Repository: Parsed transaksi {}: ID={}, Name={}, Total={}", 
                             i, transaksi.id, transaksi.nama_pelanggan, transaksi.total_harga);
                    transaksi_list.push(transaksi);
                }
                Err(e) => {
                    println!("❌ Repository: Error parsing row {}: {:?}", i, e);
                    return Err(e);
                }
            }
        }
        
        println!("✅ Repository: Successfully parsed {} transaksi", transaksi_list.len());
        Ok(transaksi_list)
    }

    pub async fn get_transaksi_by_pelanggan(mut db: PoolConnection<Any>, id_pelanggan: i32) -> Result<Vec<Transaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, 
                       total_harga::text as total_harga_str, status, catatan
                FROM transaksi
                WHERE id_pelanggan = $1
                ORDER BY tanggal_transaksi DESC
            ")
            .bind(id_pelanggan)
            .fetch_all(&mut *db)
            .await?;
        
        let mut transaksi_list = Vec::new();
        for row in rows {
            let transaksi = Self::parse_row_to_transaksi(row)?;
            transaksi_list.push(transaksi);
        }
        
        Ok(transaksi_list)
    }

    pub async fn get_transaksi_by_status(mut db: PoolConnection<Any>, status: &StatusTransaksi) -> Result<Vec<Transaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, 
                       total_harga::text as total_harga_str, status, catatan
                FROM transaksi
                WHERE status = $1
                ORDER BY tanggal_transaksi DESC
            ")
            .bind(status.to_string())
            .fetch_all(&mut *db)
            .await?;
        
        let mut transaksi_list = Vec::new();
        for row in rows {
            let transaksi = Self::parse_row_to_transaksi(row)?;
            transaksi_list.push(transaksi);
        }
        
        Ok(transaksi_list)
    }

    pub async fn create_detail_transaksi(mut db: PoolConnection<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let result = sqlx::query("
                INSERT INTO detail_transaksi (id_transaksi, id_produk, harga_satuan, jumlah, subtotal, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, id_transaksi, id_produk, harga_satuan::text as harga_satuan_str, jumlah, subtotal::text as subtotal_str
            ")
            .bind(detail.id_transaksi)
            .bind(detail.id_produk)
            .bind(detail.harga_satuan)
            .bind(detail.jumlah as i32)
            .bind(detail.subtotal)
            .bind(&now)
            .bind(&now)
            .fetch_one(&mut *db)
            .await?;
        
        let detail = Self::parse_row_to_detail_transaksi(result)?;
        Ok(detail)
    }

    pub async fn get_detail_by_transaksi_id(mut db: PoolConnection<Any>, id_transaksi: i32) -> Result<Vec<DetailTransaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_transaksi, id_produk, 
                       harga_satuan::text as harga_satuan_str, jumlah, subtotal::text as subtotal_str
                FROM detail_transaksi
                WHERE id_transaksi = $1
                ORDER BY id
            ")
            .bind(id_transaksi)
            .fetch_all(&mut *db)
            .await?;
        
        let mut detail_list = Vec::new();
        for row in rows {
            let detail = Self::parse_row_to_detail_transaksi(row)?;
            detail_list.push(detail);
        }
        
        Ok(detail_list)
    }

    pub async fn update_detail_transaksi(mut db: PoolConnection<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let result = sqlx::query("
                UPDATE detail_transaksi
                SET id_produk = $1, harga_satuan = $2, jumlah = $3, subtotal = $4, updated_at = $5
                WHERE id = $6
                RETURNING id, id_transaksi, id_produk, harga_satuan::text as harga_satuan_str, jumlah, subtotal::text as subtotal_str
            ")
            .bind(detail.id_produk)
            .bind(detail.harga_satuan)
            .bind(detail.jumlah as i32)
            .bind(detail.subtotal)
            .bind(&now)
            .bind(detail.id)
            .fetch_one(&mut *db)
            .await?;
        
        let detail = Self::parse_row_to_detail_transaksi(result)?;
        Ok(detail)
    }

    pub async fn delete_detail_transaksi(mut db: PoolConnection<Any>, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM detail_transaksi WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }

    pub async fn delete_detail_by_transaksi_id(mut db: PoolConnection<Any>, id_transaksi: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM detail_transaksi WHERE id_transaksi = $1")
            .bind(id_transaksi)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }
    
    fn parse_row_to_transaksi(row: AnyRow) -> Result<Transaksi, sqlx::Error> {
        println!("🔧 Parsing transaksi row with string conversion...");
        
        let id: i32 = row.try_get("id")?;
        let id_pelanggan: i32 = row.try_get("id_pelanggan")?;
        let nama_pelanggan: String = row.try_get("nama_pelanggan")?;
        let tanggal_transaksi: String = row.try_get("tanggal_transaksi")?;
        
        let total_harga_str: String = row.try_get("total_harga_str")?;
        let total_harga: f64 = total_harga_str.parse::<f64>().map_err(|e| {
            println!("❌ Error parsing total_harga '{}': {:?}", total_harga_str, e);
            sqlx::Error::ColumnDecode { 
                index: "total_harga_str".to_string(), 
                source: Box::new(e) 
            }
        })?;
        
        let status_str: String = row.try_get("status")?;
        let status = StatusTransaksi::from_string(&status_str).unwrap_or(StatusTransaksi::MasihDiproses);
        
        let catatan: Option<String> = row.try_get::<Option<String>, _>("catatan").unwrap_or(None);
        
        let mut transaksi = Transaksi::new(
            id_pelanggan,
            nama_pelanggan,
            total_harga,
            catatan,
        );

        transaksi.id = id;
        transaksi.tanggal_transaksi = tanggal_transaksi;
        transaksi.status = status;

        println!("✅ Successfully parsed transaksi: ID={}, Name={}, Total={}", 
                 transaksi.id, transaksi.nama_pelanggan, transaksi.total_harga);

        Ok(transaksi)
    }

    fn parse_row_to_detail_transaksi(row: AnyRow) -> Result<DetailTransaksi, sqlx::Error> {
        let id: i32 = row.try_get("id")?;
        let id_transaksi: i32 = row.try_get("id_transaksi")?;
        let id_produk: i32 = row.try_get("id_produk")?;
        
        let harga_satuan_str: String = row.try_get("harga_satuan_str")?;
        let harga_satuan: f64 = harga_satuan_str.parse::<f64>().map_err(|e| {
            sqlx::Error::ColumnDecode { 
                index: "harga_satuan_str".to_string(), 
                source: Box::new(e) 
            }
        })?;
        
        let subtotal_str: String = row.try_get("subtotal_str")?;
        let subtotal: f64 = subtotal_str.parse::<f64>().map_err(|e| {
            sqlx::Error::ColumnDecode { 
                index: "subtotal_str".to_string(), 
                source: Box::new(e) 
            }
        })?;
        
        let jumlah: u32 = row.try_get::<i32, _>("jumlah")? as u32;

        Ok(DetailTransaksi {
            id,
            id_transaksi,
            id_produk,
            harga_satuan,
            jumlah,
            subtotal,
        })
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
    async fn test_numeric_string_conversion() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            99,
            "Numeric Test".to_string(),
            1234.56,
            Some("Testing numeric conversion".to_string()),
        );

        let created = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();
        
        assert!(created.id > 0);
        assert_eq!(created.id_pelanggan, 99);
        assert_eq!(created.nama_pelanggan, "Numeric Test");
        assert!((created.total_harga - 1234.56).abs() < 0.01);
        
        println!("✅ Numeric string conversion test passed");
    }
}