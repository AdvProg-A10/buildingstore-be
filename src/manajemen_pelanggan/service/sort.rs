use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

pub trait SortStrategy {
    fn execute(&self, customers: &mut Vec<Pelanggan>);
}

pub struct SortByNama;
impl SortStrategy for SortByNama {
    fn execute(&self, customers: &mut Vec<Pelanggan>) {
        customers.sort_by(|a, b| a.nama.cmp(&b.nama));
    }
}

pub struct SortByTanggalGabung;
impl SortStrategy for SortByTanggalGabung {
    fn execute(&self, customers: &mut Vec<Pelanggan>) {
        customers.sort_by(|a, b| a.tanggal_gabung.cmp(&b.tanggal_gabung));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

    #[test]
    fn test_sort_by_nama() {
        let mut customers = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let sorter = SortByNama;
        sorter.execute(&mut customers);

        assert_eq!(customers[0].nama, "Jane Smith");
        assert_eq!(customers[1].nama, "John Doe");
    }

    #[test]
    fn test_sort_by_tanggal_gabung() {
        let mut customers = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let sorter = SortByTanggalGabung;
        sorter.execute(&mut customers);

        assert_eq!(customers[0].tanggal_gabung, customers[1].tanggal_gabung);
    }
}