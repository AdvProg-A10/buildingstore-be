pub mod dto;
pub mod create;
pub mod read;
pub mod update;
pub mod delete;

pub struct ProdukRepository;

// Export semua fungsi yang diperlukan
pub use dto::{
    RepositoryError, 
    get_db_pool_from_state,
    validate_produk,
    row_to_produk,
    get_store_stats
};

// Semua fungsi repository sekarang menerima pool sebagai parameter
pub use create::*;
pub use read::*;
pub use update::*;
pub use delete::*;