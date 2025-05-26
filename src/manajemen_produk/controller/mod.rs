use rocket::Route;
use rocket::fairing::AdHoc;

// Fungsi untuk mengembalikan semua routes
pub fn routes() -> Vec<Route> {
    let mut all_routes = Vec::new();
    
    // Gabungkan semua routes dari masing-masing modul
    all_routes.extend(create::routes());
    all_routes.extend(read::routes());
    all_routes.extend(update::routes());
    all_routes.extend(delete::routes());
    
    all_routes
}

// Route stage untuk digunakan di main.rs
pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Manajemen Produk Routes", |rocket| async {
        rocket.mount("/api", routes())
    })
}

pub mod create;
pub mod read;
pub mod update;
pub mod delete;
pub mod dto;

// Re-export untuk kemudahan akses
pub use dto::*;