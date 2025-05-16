#[derive(Debug, Clone, PartialEq)]
pub enum StatusTransaksi {
    MasihDiproses,
    Selesai,
    Dibatalkan,
}

impl StatusTransaksi {
    pub fn from_string(input: &str) -> Option<Self> {
        match input.to_uppercase().as_str() {
            "MASIHDIPROSES" => Some(StatusTransaksi::MasihDiproses),
            "SELESAI" => Some(StatusTransaksi::Selesai),
            "DIBATALKAN" => Some(StatusTransaksi::Dibatalkan),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            StatusTransaksi::MasihDiproses => "MASIHDIPROSES".to_string(),
            StatusTransaksi::Selesai => "SELESAI".to_string(),
            StatusTransaksi::Dibatalkan => "DIBATALKAN".to_string(),
        }
    }
}
