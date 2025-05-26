use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatusTransaksi {
    MasihDiproses,
    Selesai,
    Dibatalkan,
}

impl StatusTransaksi {
    pub fn from_string(status: &str) -> Option<Self> {
        match status.to_uppercase().as_str() {
            "MASIH_DIPROSES" | "MASIH DIPROSES" | "DIPROSES" => Some(StatusTransaksi::MasihDiproses),
            "SELESAI" | "COMPLETED" | "DONE" => Some(StatusTransaksi::Selesai),
            "DIBATALKAN" | "CANCELLED" | "BATAL" => Some(StatusTransaksi::Dibatalkan),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            StatusTransaksi::MasihDiproses => "MASIH_DIPROSES".to_string(),
            StatusTransaksi::Selesai => "SELESAI".to_string(),
            StatusTransaksi::Dibatalkan => "DIBATALKAN".to_string(),
        }
    }

    pub fn can_be_modified(&self) -> bool {
        matches!(self, StatusTransaksi::MasihDiproses)
    }

    pub fn can_be_cancelled(&self) -> bool {
        matches!(self, StatusTransaksi::MasihDiproses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_transaksi_from_string() {
        let masih_diproses = StatusTransaksi::from_string("MASIH_DIPROSES");
        assert!(masih_diproses.is_some());
        assert_eq!(masih_diproses.unwrap(), StatusTransaksi::MasihDiproses);

        let selesai = StatusTransaksi::from_string("SELESAI");
        assert!(selesai.is_some());
        assert_eq!(selesai.unwrap(), StatusTransaksi::Selesai);

        let dibatalkan = StatusTransaksi::from_string("DIBATALKAN");
        assert!(dibatalkan.is_some());
        assert_eq!(dibatalkan.unwrap(), StatusTransaksi::Dibatalkan);

        let lowercase_status = StatusTransaksi::from_string("selesai");
        assert!(lowercase_status.is_some());
        assert_eq!(lowercase_status.unwrap(), StatusTransaksi::Selesai);

        let alternative = StatusTransaksi::from_string("COMPLETED");
        assert!(alternative.is_some());
        assert_eq!(alternative.unwrap(), StatusTransaksi::Selesai);

        let invalid_status = StatusTransaksi::from_string("INVALID");
        assert!(invalid_status.is_none());
    }

    #[test]
    fn test_status_transaksi_to_string() {
        assert_eq!(StatusTransaksi::MasihDiproses.to_string(), "MASIH_DIPROSES");
        assert_eq!(StatusTransaksi::Selesai.to_string(), "SELESAI");
        assert_eq!(StatusTransaksi::Dibatalkan.to_string(), "DIBATALKAN");
    }

    #[test]
    fn test_can_be_modified() {
        assert!(StatusTransaksi::MasihDiproses.can_be_modified());
        assert!(!StatusTransaksi::Selesai.can_be_modified());
        assert!(!StatusTransaksi::Dibatalkan.can_be_modified());
    }

    #[test]
    fn test_can_be_cancelled() {
        assert!(StatusTransaksi::MasihDiproses.can_be_cancelled());
        assert!(!StatusTransaksi::Selesai.can_be_cancelled());
        assert!(!StatusTransaksi::Dibatalkan.can_be_cancelled());
    }
}
