use crate::transaksi_penjualan::model::transaksi::Transaksi;

pub trait SortingStrategy: Send + Sync {
    fn sort(&self, transaksi_list: Vec<Transaksi>) -> Vec<Transaksi>;
    fn get_name(&self) -> &'static str;
}

pub struct SortByDate;
impl SortingStrategy for SortByDate {
    fn sort(&self, mut transaksi_list: Vec<Transaksi>) -> Vec<Transaksi> {
        transaksi_list.sort_by(|a, b| a.tanggal_transaksi.cmp(&b.tanggal_transaksi));
        transaksi_list
    }
    
    fn get_name(&self) -> &'static str { "date_asc" }
}

pub struct SortByDateDesc;
impl SortingStrategy for SortByDateDesc {
    fn sort(&self, mut transaksi_list: Vec<Transaksi>) -> Vec<Transaksi> {
        transaksi_list.sort_by(|a, b| b.tanggal_transaksi.cmp(&a.tanggal_transaksi));
        transaksi_list
    }
    
    fn get_name(&self) -> &'static str { "date_desc" }
}

pub struct SortByTotal;
impl SortingStrategy for SortByTotal {
    fn sort(&self, mut transaksi_list: Vec<Transaksi>) -> Vec<Transaksi> {
        transaksi_list.sort_by(|a, b| a.total_harga.partial_cmp(&b.total_harga).unwrap());
        transaksi_list
    }
    
    fn get_name(&self) -> &'static str { "total_asc" }
}

pub struct SortByTotalDesc;
impl SortingStrategy for SortByTotalDesc {
    fn sort(&self, mut transaksi_list: Vec<Transaksi>) -> Vec<Transaksi> {
        transaksi_list.sort_by(|a, b| b.total_harga.partial_cmp(&a.total_harga).unwrap());
        transaksi_list
    }
    
    fn get_name(&self) -> &'static str { "total_desc" }
}

pub struct SortByCustomer;
impl SortingStrategy for SortByCustomer {
    fn sort(&self, mut transaksi_list: Vec<Transaksi>) -> Vec<Transaksi> {
        transaksi_list.sort_by(|a, b| a.nama_pelanggan.cmp(&b.nama_pelanggan));
        transaksi_list
    }
    
    fn get_name(&self) -> &'static str { "customer_asc" }
}

pub struct SortByStatus;
impl SortingStrategy for SortByStatus {
    fn sort(&self, mut transaksi_list: Vec<Transaksi>) -> Vec<Transaksi> {
        transaksi_list.sort_by(|a, b| a.status.to_string().cmp(&b.status.to_string()));
        transaksi_list
    }
    
    fn get_name(&self) -> &'static str { "status" }
}

// Factory untuk sorting strategy
pub struct SortingStrategyFactory;

impl SortingStrategyFactory {
    pub fn create(sort_type: &str) -> Box<dyn SortingStrategy> {
        match sort_type.to_lowercase().as_str() {
            "tanggal" | "tanggal_transaksi" | "date_asc" => Box::new(SortByDate),
            "tanggal_desc" | "date_desc" => Box::new(SortByDateDesc),
            "total" | "total_harga" | "total_asc" => Box::new(SortByTotal),
            "total_desc" => Box::new(SortByTotalDesc),
            "pelanggan" | "nama_pelanggan" | "customer" => Box::new(SortByCustomer),
            "status" => Box::new(SortByStatus),
            _ => Box::new(SortByDateDesc), // Default
        }
    }

    pub fn get_available_strategies() -> Vec<&'static str> {
        vec!["date_asc", "date_desc", "total_asc", "total_desc", "customer_asc", "status"]
    }
}

// Strategy untuk filtering
pub trait FilteringStrategy: Send + Sync {
    fn filter(&self, transaksi_list: Vec<Transaksi>, keyword: &str) -> Vec<Transaksi>;
    fn get_name(&self) -> &'static str;
}

pub struct FilterByCustomer;
impl FilteringStrategy for FilterByCustomer {
    fn filter(&self, transaksi_list: Vec<Transaksi>, keyword: &str) -> Vec<Transaksi> {
        let keyword = keyword.to_lowercase();
        transaksi_list.into_iter()
            .filter(|t| t.nama_pelanggan.to_lowercase().contains(&keyword))
            .collect()
    }
    
    fn get_name(&self) -> &'static str { "customer" }
}

pub struct FilterByStatus;
impl FilteringStrategy for FilterByStatus {
    fn filter(&self, transaksi_list: Vec<Transaksi>, keyword: &str) -> Vec<Transaksi> {
        let keyword = keyword.to_lowercase();
        transaksi_list.into_iter()
            .filter(|t| t.status.to_string().to_lowercase().contains(&keyword))
            .collect()
    }
    
    fn get_name(&self) -> &'static str { "status" }
}

pub struct FilterByNote;
impl FilteringStrategy for FilterByNote {
    fn filter(&self, transaksi_list: Vec<Transaksi>, keyword: &str) -> Vec<Transaksi> {
        let keyword = keyword.to_lowercase();
        transaksi_list.into_iter()
            .filter(|t| {
                if let Some(ref catatan) = t.catatan {
                    catatan.to_lowercase().contains(&keyword)
                } else {
                    false
                }
            })
            .collect()
    }
    
    fn get_name(&self) -> &'static str { "note" }
}

pub struct FilterByAll;
impl FilteringStrategy for FilterByAll {
    fn filter(&self, transaksi_list: Vec<Transaksi>, keyword: &str) -> Vec<Transaksi> {
        let keyword = keyword.to_lowercase();
        transaksi_list.into_iter()
            .filter(|t| {
                t.nama_pelanggan.to_lowercase().contains(&keyword) ||
                t.status.to_string().to_lowercase().contains(&keyword) ||
                t.id.to_string().contains(&keyword) ||
                (t.catatan.as_ref().map_or(false, |c| c.to_lowercase().contains(&keyword)))
            })
            .collect()
    }
    
    fn get_name(&self) -> &'static str { "all" }
}

pub struct FilteringStrategyFactory;

impl FilteringStrategyFactory {
    pub fn create(filter_type: &str) -> Box<dyn FilteringStrategy> {
        match filter_type.to_lowercase().as_str() {
            "pelanggan" | "nama_pelanggan" | "customer" => Box::new(FilterByCustomer),
            "status" => Box::new(FilterByStatus),
            "catatan" | "note" => Box::new(FilterByNote),
            _ => Box::new(FilterByAll), // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

    fn create_test_data() -> Vec<Transaksi> {
        vec![
            Transaksi {
                id: 1,
                id_pelanggan: 1,
                nama_pelanggan: "Charlie".to_string(),
                tanggal_transaksi: "2024-01-15 10:00:00".to_string(),
                total_harga: 150000.0,
                status: StatusTransaksi::MasihDiproses,
                catatan: Some("Test 1".to_string()),
            },
            Transaksi {
                id: 2,
                id_pelanggan: 2,
                nama_pelanggan: "Alice".to_string(),
                tanggal_transaksi: "2024-01-10 14:30:00".to_string(),
                total_harga: 250000.0,
                status: StatusTransaksi::Selesai,
                catatan: Some("Test 2".to_string()),
            },
            Transaksi {
                id: 3,
                id_pelanggan: 3,
                nama_pelanggan: "Bob".to_string(),
                tanggal_transaksi: "2024-01-20 09:15:00".to_string(),
                total_harga: 100000.0,
                status: StatusTransaksi::Dibatalkan,
                catatan: None,
            },
        ]
    }

    #[test]
    fn test_sorting_strategies() {
        let data = create_test_data();

        let sort_date = SortByDate;
        let sorted = sort_date.sort(data.clone());
        assert_eq!(sorted[0].tanggal_transaksi, "2024-01-10 14:30:00");
        assert_eq!(sorted[2].tanggal_transaksi, "2024-01-20 09:15:00");

        let sort_total_desc = SortByTotalDesc;
        let sorted = sort_total_desc.sort(data.clone());
        assert_eq!(sorted[0].total_harga, 250000.0);
        assert_eq!(sorted[2].total_harga, 100000.0);

        let sort_customer = SortByCustomer;
        let sorted = sort_customer.sort(data);
        assert_eq!(sorted[0].nama_pelanggan, "Alice");
        assert_eq!(sorted[1].nama_pelanggan, "Bob");
        assert_eq!(sorted[2].nama_pelanggan, "Charlie");
    }

    #[test]
    fn test_filtering_strategies() {
        let data = create_test_data();

        let filter_customer = FilterByCustomer;
        let filtered = filter_customer.filter(data.clone(), "Alice");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].nama_pelanggan, "Alice");

        let filter_status = FilterByStatus;
        let filtered = filter_status.filter(data.clone(), "SELESAI");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, StatusTransaksi::Selesai);

        let filter_all = FilterByAll;
        let filtered = filter_all.filter(data, "Test");
        assert_eq!(filtered.len(), 2); // Should find "Test 1" and "Test 2"
    }

    #[test]
    fn test_strategy_factories() {
        let data = create_test_data();

        let sort_strategy = SortingStrategyFactory::create("total_desc");
        let sorted = sort_strategy.sort(data.clone());
        assert_eq!(sorted[0].total_harga, 250000.0);

        let filter_strategy = FilteringStrategyFactory::create("customer");
        let filtered = filter_strategy.filter(data, "Alice");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_strategy_combination() {
        let data = create_test_data();

        let filter_strategy = FilteringStrategyFactory::create("status");
        let filtered = filter_strategy.filter(data, "MASIH_DIPROSES");

        let sort_strategy = SortingStrategyFactory::create("total_asc");
        let result = sort_strategy.sort(filtered);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, StatusTransaksi::MasihDiproses);
    }

    #[test]
    fn test_empty_data_handling() {
        let empty_data: Vec<Transaksi> = vec![];

        let sort_strategy = SortByTotal;
        let sorted = sort_strategy.sort(empty_data.clone());
        assert_eq!(sorted.len(), 0);

        let filter_strategy = FilterByCustomer;
        let filtered = filter_strategy.filter(empty_data, "anything");
        assert_eq!(filtered.len(), 0);
    }
}