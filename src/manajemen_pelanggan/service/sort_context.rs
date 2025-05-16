use crate::manajemen_pelanggan::service::sort::SortStrategy;
use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

use super::sort::SortByNama;

pub struct SortContext<'a> {
    sort_strategy: Box<dyn SortStrategy + 'a>,
}

impl<'a> SortContext<'a> {
    pub fn new() -> Self {
        SortContext {
            sort_strategy: Box::new(SortByNama),
        }
    }

    pub fn set_strategy(&mut self, sort_strategy: Box<dyn SortStrategy>) {
        self.sort_strategy = sort_strategy;
    }

    pub fn execute_sort(&mut self, customers: &mut Vec<Pelanggan>) {
        self.sort_strategy.execute(customers);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;
    use crate::manajemen_pelanggan::service::sort::SortByNama;

    #[test]
    fn test_sort_context() {
        let mut customers = vec![
            Pelanggan::new("John Doe".to_string(), "123 Main St".to_string(), "1234567890".to_string()),
            Pelanggan::new("Jane Smith".to_string(), "456 Elm St".to_string(), "0987654321".to_string()),
        ];

        let mut sort_context = SortContext::new();
        sort_context.set_strategy(Box::new(SortByNama));

        sort_context.execute_sort(&mut customers);

        assert_eq!(customers[0].nama, "Jane Smith");
        assert_eq!(customers[1].nama, "John Doe");
    }
}