use chrono::{Utc, NaiveDateTime};
use rocket::serde::{Serialize, Deserialize};
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Transaksi {
    pub id: i32,
    pub id_pelanggan: i32,
    pub nama_pelanggan: String,
    pub tanggal_transaksi: String,  
    pub total_harga: f64,
    pub status: StatusTransaksi,
    pub catatan: Option<String>,
}

impl Transaksi {
    pub fn new(
        id_pelanggan: i32,
        nama_pelanggan: String,
        total_harga: f64,
        catatan: Option<String>,
    ) -> Self {
        Transaksi {
            id: 0,
            id_pelanggan,
            nama_pelanggan,
            tanggal_transaksi: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_harga,
            status: StatusTransaksi::MasihDiproses,
            catatan,
        }
    }

    pub fn can_be_modified(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }

    pub fn can_be_cancelled(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }

    pub fn can_add_items(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }

    pub fn can_update_items(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }

    pub fn can_delete_items(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }

    pub fn can_be_completed(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }

    pub fn complete(&mut self) -> Result<(), String> {
        if !self.can_be_completed() {
            return Err("Transaksi tidak dapat diselesaikan".to_string());
        }
        self.status = StatusTransaksi::Selesai;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        if !self.can_be_cancelled() {
            return Err("Transaksi tidak dapat dibatalkan".to_string());
        }
        self.status = StatusTransaksi::Dibatalkan;
        Ok(())
    }

    pub fn reopen(&mut self) -> Result<(), String> {
        if self.status == StatusTransaksi::MasihDiproses {
            return Err("Transaksi sudah dalam status diproses".to_string());
        }
        self.status = StatusTransaksi::MasihDiproses;
        Ok(())
    }

    pub fn update_status(&mut self, status: StatusTransaksi) {
        self.status = status;
    }

    pub fn update_total_harga(&mut self, total_harga: f64) {
        self.total_harga = total_harga;
    }

    pub fn get_allowed_actions(&self) -> Vec<String> {
        match self.status {
            StatusTransaksi::MasihDiproses => vec![
                "complete".to_string(),
                "cancel".to_string(),
                "add_item".to_string(),
                "update_item".to_string(),
                "delete_item".to_string(),
            ],
            StatusTransaksi::Selesai => vec![
                "print_receipt".to_string(),
                "view_details".to_string(),
                "reopen".to_string(), 
            ],
            StatusTransaksi::Dibatalkan => vec![
                "view_details".to_string(),
                "reopen".to_string(), 
            ],
        }
    }

    pub fn get_tanggal_as_datetime(&self) -> Result<NaiveDateTime, chrono::ParseError> {
        NaiveDateTime::parse_from_str(&self.tanggal_transaksi, "%Y-%m-%d %H:%M:%S")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_transaksi() {
        let transaksi = Transaksi::new(
            1,
            "Castorice".to_string(),
            150000.0,
            Some("Pembelian produk elektronik".to_string()),
        );

        assert_eq!(transaksi.id_pelanggan, 1);
        assert_eq!(transaksi.nama_pelanggan, "Castorice");
        assert_eq!(transaksi.total_harga, 150000.0);
        assert_eq!(transaksi.status, StatusTransaksi::MasihDiproses);
        assert!(transaksi.can_be_modified());
    }

    #[test]
    fn test_update_status() {
        let mut transaksi = Transaksi::new(
            1,
            "Tribbie".to_string(),
            200000.0,
            None,
        );

        transaksi.update_status(StatusTransaksi::Selesai);
        assert_eq!(transaksi.status, StatusTransaksi::Selesai);
        assert!(!transaksi.can_be_modified());
    }

    #[test]
    fn test_state_transitions() {
        let mut transaksi = Transaksi::new(
            2,
            "Hyacine".to_string(),
            100000.0,
            None,
        );

        assert!(transaksi.complete().is_ok());
        assert_eq!(transaksi.status, StatusTransaksi::Selesai);
        assert!(!transaksi.can_be_modified());

        assert!(transaksi.reopen().is_ok());
        assert_eq!(transaksi.status, StatusTransaksi::MasihDiproses);
        assert!(transaksi.can_be_modified());

        assert!(transaksi.cancel().is_ok());
        assert_eq!(transaksi.status, StatusTransaksi::Dibatalkan);
        assert!(!transaksi.can_be_modified());
    }

    #[test]
    fn test_permissions() {
        let mut transaksi = Transaksi::new(
            1,
            "Test".to_string(),
            100000.0,
            None,
        );

        assert!(transaksi.can_be_modified());
        assert!(transaksi.can_add_items());
        assert!(transaksi.can_be_cancelled());
        assert!(transaksi.can_be_completed());

        transaksi.complete().unwrap();
        assert!(!transaksi.can_be_modified());
        assert!(!transaksi.can_add_items());
        assert!(!transaksi.can_be_cancelled());
        assert!(!transaksi.can_be_completed());

        let actions = transaksi.get_allowed_actions();
        assert!(actions.contains(&"print_receipt".to_string()));
        assert!(!actions.contains(&"add_item".to_string()));
    }

    #[test]
    fn test_update_total_harga() {
        let mut transaksi = Transaksi::new(
            2,
            "Hyacine".to_string(),
            100000.0,
            None,
        );

        transaksi.update_total_harga(175000.0);
        assert_eq!(transaksi.total_harga, 175000.0);
    }

    #[test]
    fn test_datetime_parsing() {
        let transaksi = Transaksi::new(
            1,
            "Test DateTime".to_string(),
            100000.0,
            None,
        );

        let parsed_datetime = transaksi.get_tanggal_as_datetime();
        assert!(parsed_datetime.is_ok());
    }
}