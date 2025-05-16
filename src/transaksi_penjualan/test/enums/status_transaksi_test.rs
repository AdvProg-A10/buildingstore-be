#[cfg(test)]
mod tests {
    use crate::transaksi_penjualan::main::enums::status_transaksi::StatusTransaksi;

    #[test]
    fn test_status_transaksi_from_string() {
        assert_eq!(
            StatusTransaksi::from_string("masihdiproses"),
            Some(StatusTransaksi::MasihDiproses)
        );
        assert_eq!(
            StatusTransaksi::from_string("SELESAI"),
            Some(StatusTransaksi::Selesai)
        );
        assert_eq!(
            StatusTransaksi::from_string("dibatalkan"),
            Some(StatusTransaksi::Dibatalkan)
        );
        assert_eq!(StatusTransaksi::from_string("invalid"), None);
    }

    #[test]
    fn test_status_transaksi_to_string() {
        assert_eq!(StatusTransaksi::MasihDiproses.to_string(), "MASIHDIPROSES");
        assert_eq!(StatusTransaksi::Selesai.to_string(), "SELESAI");
        assert_eq!(StatusTransaksi::Dibatalkan.to_string(), "DIBATALKAN");
    }
}
