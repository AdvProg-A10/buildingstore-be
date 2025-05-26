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
