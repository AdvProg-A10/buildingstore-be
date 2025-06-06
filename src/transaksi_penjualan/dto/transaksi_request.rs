use rocket::serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTransaksiRequest {
    pub id_pelanggan: i32,
    pub nama_pelanggan: String,
    pub catatan: Option<String>,
    pub detail_transaksi: Vec<CreateDetailTransaksiRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateDetailTransaksiRequest {
    pub id_produk: i32,
    pub nama_produk: String,
    pub harga_satuan: f64,
    pub jumlah: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateDetailQuantityRequest {
    pub jumlah: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TransaksiWithDetailsResponse {
    pub id: i32,
    pub id_pelanggan: i32,
    pub nama_pelanggan: String,
    pub tanggal_transaksi: String,
    pub total_harga: f64,
    pub status: String,
    pub catatan: Option<String>,
    pub detail_transaksi: Vec<DetailTransaksi>,
}

impl CreateTransaksiRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.nama_pelanggan.trim().is_empty() {
            return Err("Customer name cannot be empty".to_string());
        }

        if self.id_pelanggan <= 0 {
            return Err("Invalid customer ID".to_string());
        }

        if self.detail_transaksi.is_empty() {
            return Err("Transaction must have at least one product".to_string());
        }

        for (index, detail) in self.detail_transaksi.iter().enumerate() {
            if let Err(err) = detail.validate() {
                return Err(format!("Detail {}: {}", index + 1, err));
            }
        }

        Ok(())
    }

    pub fn calculate_total(&self, product_prices: &HashMap<i32, f64>) -> f64 {
        self.detail_transaksi
            .iter()
            .map(|detail| {
                let price = product_prices.get(&detail.id_produk).unwrap_or(&detail.harga_satuan);
                price * detail.jumlah as f64
            })
            .sum()
    }
}

impl CreateDetailTransaksiRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.nama_produk.trim().is_empty() {
            return Err("Product name cannot be empty".to_string());
        }

        if self.id_produk <= 0 {
            return Err("Invalid product ID".to_string());
        }

        if self.jumlah == 0 {
            return Err("Quantity must be greater than 0".to_string());
        }

        if self.harga_satuan < 0.0 {
            return Err("Unit price cannot be negative".to_string());
        }

        Ok(())
    }

    pub fn to_detail_transaksi(&self, id_transaksi: i32, harga_satuan: f64) -> DetailTransaksi {
        DetailTransaksi::new(
            id_transaksi,
            self.id_produk,
            harga_satuan,
            self.jumlah,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_detail_transaksi_request_to_detail_transaksi() {
        let request = CreateDetailTransaksiRequest {
            id_produk: 101,
            nama_produk: "Macbook Pro M3".to_string(),
            harga_satuan: 15000000.0,
            jumlah: 2,
        };

        let detail = request.to_detail_transaksi(1, request.harga_satuan);

        assert_eq!(detail.id_transaksi, 1);
        assert_eq!(detail.id_produk, 101);
        assert_eq!(detail.harga_satuan, 15000000.0);
        assert_eq!(detail.jumlah, 2);
        assert_eq!(detail.subtotal, 30000000.0);
    }

    #[test]
    fn test_validate_valid_request() {
        let request = CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Alice".to_string(),
            catatan: Some("Test".to_string()),
            detail_transaksi: vec![
                CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Produk A".to_string(),
                    harga_satuan: 10000.0,
                    jumlah: 2,
                },
            ],
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_request_empty_detail() {
        let request = CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Alice".to_string(),
            catatan: None,
            detail_transaksi: vec![],
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_negative_price() {
        let request = CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Alice".to_string(),
            catatan: None,
            detail_transaksi: vec![
                CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Produk A".to_string(),
                    harga_satuan: -100.0,
                    jumlah: 2,
                },
            ],
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_calculate_total() {
        let request = CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Alice".to_string(),
            catatan: None,
            detail_transaksi: vec![
                CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Produk A".to_string(),
                    harga_satuan: 0.0,
                    jumlah: 3,
                },
                CreateDetailTransaksiRequest {
                    id_produk: 2,
                    nama_produk: "Produk B".to_string(),
                    harga_satuan: 0.0,
                    jumlah: 2,
                },
            ],
        };

        let mut product_prices = HashMap::new();
        product_prices.insert(1, 10000.0);
        product_prices.insert(2, 20000.0);

        let total = request.calculate_total(&product_prices);
        assert_eq!(total, 3.0 * 10000.0 + 2.0 * 20000.0);
    }
}