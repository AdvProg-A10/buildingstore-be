use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

pub trait FilterStrategy {
    fn execute(&self, pelanggan_vec: &mut Vec<Pelanggan>, query: &str);
}

pub struct FilterByNama;
impl FilterStrategy for FilterByNama {
    fn execute(&self, pelanggan_vec: &mut Vec<Pelanggan>, query: &str) {
        pelanggan_vec.retain(|customer| customer.nama.contains(query));
    }
}

pub struct FilterByTanggalGabungPrev;
impl FilterStrategy for FilterByTanggalGabungPrev {
    fn execute(&self, pelanggan_vec: &mut Vec<Pelanggan>, query: &str) {
        let date = match chrono::NaiveDate::parse_from_str(query, "%Y-%m-%d") {
            Ok(parsed_date) => parsed_date,
            Err(_) => return,
        };
        pelanggan_vec.retain(|customer| {customer.tanggal_gabung < date});
    }
}

pub struct FilterByTanggalGabungAfter;
impl FilterStrategy for FilterByTanggalGabungAfter {
    fn execute(&self, pelanggan_vec: &mut Vec<Pelanggan>, query: &str) {
        let date = match chrono::NaiveDate::parse_from_str(query, "%Y-%m-%d") {
            Ok(parsed_date) => parsed_date,
            Err(_) => return,
        };
        pelanggan_vec.retain(|customer| {customer.tanggal_gabung > date});
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

    #[test]
    fn test_filter_by_nama() {
        let mut pelanggan_vec = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let filter = FilterByNama;
        filter.execute(&mut pelanggan_vec, "John");

        assert_eq!(pelanggan_vec.len(), 1);
        assert_eq!(pelanggan_vec[0].nama, "John Doe");
    }

    
    #[test]
    fn test_filter_by_tanggal_gabung_prev() {
        let mut pelanggan_vec = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let filter = FilterByTanggalGabungPrev;
        filter.execute(&mut pelanggan_vec, "2023-01-01");

        assert_eq!(pelanggan_vec.len(), 0);
    }

    #[test]
    fn test_filter_by_tanggal_gabung_after() {
        let mut pelanggan_vec = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let filter = FilterByTanggalGabungAfter;
        filter.execute(&mut pelanggan_vec, "2023-01-01");

        assert_eq!(pelanggan_vec.len(), 2);
    }
}