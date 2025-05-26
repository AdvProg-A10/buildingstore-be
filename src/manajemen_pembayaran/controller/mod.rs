use rocket::fairing::AdHoc;

pub mod payment_controller;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Manajemen Pembayaran Routes", |rocket| async {
        rocket.mount("/api", payment_controller::routes())
    })
}