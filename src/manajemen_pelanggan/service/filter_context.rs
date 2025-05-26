use crate::manajemen_pelanggan::service::filter::FilterStrategy;
use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;
use crate::manajemen_pelanggan::service::filter::FilterByNama;

pub struct FilterContext<'a> {
    filter_strategy: Box<dyn FilterStrategy + 'a>,
}

impl Default for FilterContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterContext<'_> {
    pub fn new() -> Self {
        FilterContext {
            filter_strategy: Box::new(FilterByNama),
        }
    }

    pub fn set_strategy(&mut self, filter_strategy: Box<dyn FilterStrategy>) {
        self.filter_strategy = filter_strategy;
    }

    pub fn execute_filter(&mut self, customers: &mut Vec<Pelanggan>, query: &str) {
        self.filter_strategy.execute( customers, query);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;
    use crate::manajemen_pelanggan::service::filter::FilterByNama;

    #[test]
    fn test_sort_context() {
        let mut customers = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let mut filter_context = FilterContext::new();
        filter_context.set_strategy(Box::new(FilterByNama));

        let query = "Jane";

        filter_context.execute_filter(&mut customers, query);

        assert_eq!(customers.len(), 1);
        assert_eq!(customers[0].nama, "Jane Smith");
    }
}