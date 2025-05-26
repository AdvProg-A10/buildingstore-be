use rocket::{fairing::AdHoc, routes};

pub mod transaksi;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Initializing Transaksi routes...", |rocket| async {
        rocket.mount(
            "/api/transaksi",
            routes![
                // Basic CRUD operations
                transaksi::get_all_transaksi,
                transaksi::get_transaksi_by_id,
                transaksi::create_transaksi,
                transaksi::update_transaksi,
                transaksi::delete_transaksi,
                
                // Status operations
                transaksi::complete_transaksi,
                transaksi::cancel_transaksi,
                
                // Detail operations
                transaksi::get_detail_transaksi,
                transaksi::add_detail_transaksi,
                transaksi::update_detail_transaksi,
                transaksi::delete_detail_transaksi,
                
                // Additional operations
                transaksi::get_transaksi_with_details,
                transaksi::validate_product_stock
            ],
        )
    })
}
