use rocket::{fairing::AdHoc, routes};

pub mod auth;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Initializing /api/auth controller routes...", |rocket| async {
        rocket
            .mount("/api/auth", routes![auth::login, auth::register, auth::logout, auth::change_password, auth::get_user])
    })
}
