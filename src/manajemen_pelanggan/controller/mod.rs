use rocket::{fairing::AdHoc, routes};

pub mod pelanggan;

pub fn route_stage() -> AdHoc {
    return AdHoc::on_ignite("Initializing Pelanggan controller routes...", |rocket| async {
        rocket
            .mount("/api", routes![pelanggan::get_all_pelanggan, pelanggan::create_pelanggan, 
            pelanggan::get_pelanggan_by_id, pelanggan::update_pelanggan, pelanggan::delete_pelanggan])
    });
}
