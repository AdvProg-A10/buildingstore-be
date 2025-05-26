use rocket::{fairing::AdHoc, routes};

pub mod transaksi;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Initializing Transaksi routes...", |rocket| async {
        rocket.mount(
            "/api/transaksi",
            routes![
                transaksi::get_all_transaksi,
                transaksi::get_transaksi_by_id,
                transaksi::create_transaksi,
                transaksi::update_transaksi,
                transaksi::delete_transaksi
            ],
        )
    })
}
