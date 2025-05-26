use rocket::{get, post, patch, delete};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::{Any, Pool};
use rocket::serde::{Serialize, Deserialize};
use autometrics::autometrics;

use crate::auth::guards::auth::AuthenticatedUser;
use crate::manajemen_pelanggan::model::pelanggan::{Pelanggan, PelangganForm};
use crate::manajemen_pelanggan::service::pelanggan::PelangganService;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    message: String,
}

#[autometrics]
#[get("/pelanggan?<sort>&<filter>&<keyword>")]
pub async fn get_all_pelanggan(_user: AuthenticatedUser, db: &State<Pool<Any>>, sort: Option<String>, filter: Option<String>, keyword: Option<String>) -> Result<Json<Vec<Pelanggan>>, (Status, Json<Response>)> {
    let pelanggan = PelangganService::get_all_pelanggan(db.inner().clone()).await;
    if pelanggan.is_err() {
        return Err((Status::InternalServerError, Json(Response { message: "Failed to fetch pelanggan".to_string() })));
    }
    let mut pelanggan = pelanggan.unwrap().clone();
    if let Some(sort_strategy) = &sort {
        pelanggan = PelangganService::sort_pelanggan(pelanggan, sort_strategy);
    }
    if let (Some(filter_strategy), Some(keyword_value)) = (&filter, &keyword) {
        pelanggan = PelangganService::filter_pelanggan(pelanggan, filter_strategy, keyword_value);
    }
    Ok(Json(pelanggan))
}

#[autometrics]
#[post("/pelanggan", data = "<pelanggan>")]
pub async fn create_pelanggan(_user: AuthenticatedUser, db: &State<Pool<Any>>, pelanggan: Json<PelangganForm>) -> Result<Json<Response>, (Status, Json<Response>)> {
    let pelanggan = Pelanggan::new(pelanggan.nama.clone(), pelanggan.alamat.clone(), pelanggan.no_telp.clone());
    let res = PelangganService::create_pelanggan(db.inner().clone(), &pelanggan).await;
    if res.is_err() {
        return Err((Status::InternalServerError, Json(Response { message: "Failed to create pelanggan".to_string() })));
    }
    Ok(Json(Response { message: "Pelanggan created successfully".to_string() }))
}

#[autometrics]
#[get("/pelanggan/<id>")]
pub async fn get_pelanggan_by_id(_user: AuthenticatedUser, db: &State<Pool<Any>>, id: i32) -> Result<Json<Pelanggan>, Status> {
    let pelanggan = PelangganService::get_pelanggan_by_id(db.inner().clone(), id).await.map_err(|_| Status::NotFound)?;
    Ok(Json(pelanggan))
}

#[autometrics]
#[patch("/pelanggan/<id>", data = "<pelanggan>")]
pub async fn update_pelanggan(_user: AuthenticatedUser, db: &State<Pool<Any>>, id: i32, pelanggan: Json<Pelanggan>) -> (Status, Json<Response>) {
    if pelanggan.id != id {
        return (Status::BadRequest, Json(Response { message: "Invalid data".to_string() }));
    }
    let res = PelangganService::update_pelanggan(db.inner().clone(), &pelanggan).await;
    match res {
        Ok(_) => (Status::Ok, Json(Response { message: "Pelanggan updated successfully".to_string() })),
        Err(_) => (Status::InternalServerError, Json(Response { message: "Try again later".to_string() }))
    }
}

#[autometrics]
#[delete("/pelanggan/<id>")]
pub async fn delete_pelanggan(_user: AuthenticatedUser, db: &State<Pool<Any>>, id: i32) -> (Status, Json<Response>) {
    let res = PelangganService::delete_pelanggan(db.inner().clone(), id).await;
    if res.is_err() {
        return (Status::InternalServerError, Json(Response { message: "Failed to delete pelanggan".to_string() }));
    }
    (Status::Ok, Json(Response { message: "Pelanggan deleted successfully".to_string() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{routes, uri, async_test};
    use sqlx::any::install_default_drivers;
    use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;
    use crate::auth::model::user::User;
    use crate::auth::service::auth::AuthService;
    use crate::auth::controller::auth::*;

    const ADMIN_USERNAME: &str = "admin";
    const ADMIN_PASSWORD: &str = "admin123";

    async fn setup() -> Client {
        install_default_drivers();
        let db = sqlx::any::AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("migrations/test")
            .run(&db)
            .await
            .unwrap();

        AuthService::register_user(db.clone(), User::new(ADMIN_USERNAME.to_string(), ADMIN_PASSWORD.to_string(), true))
            .await.unwrap();

        let production = false;

        let rocket = rocket::build()
            .manage(db.clone())
            .manage(production)
            .mount("/", routes![get_all_pelanggan, create_pelanggan, 
            get_pelanggan_by_id, update_pelanggan, delete_pelanggan,
            login, register]);

        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");
        client.post(uri!(login))
            .json(&AuthForm { username: ADMIN_USERNAME.to_string(), password: ADMIN_PASSWORD.to_string() })
            .dispatch()
            .await;

        client
    }

    #[async_test]
    async fn test_create_pelanggan() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string() 
        };
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Response>().await.unwrap();
        assert_eq!(body.message, "Pelanggan created successfully");
    }

    #[async_test]
    async fn test_get_all_pelanggan() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let new_pelanggan_2 = PelangganForm { 
            nama: "Tribbie".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "1234567890".to_string()
        };

        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_2)
            .dispatch()
            .await;
        let response = client.get(uri!(super::get_all_pelanggan(_, _, _)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Pelanggan>>().await.unwrap();
        assert_eq!(body.len(), 2);
    }

    #[async_test]
    async fn test_get_pelanggan_by_id() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let response = client.get(uri!(super::get_pelanggan_by_id(1)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Pelanggan>().await.unwrap();
        assert_eq!(body.nama, new_pelanggan.nama);
    }

    #[async_test]
    async fn test_update_pelanggan() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let response = client.get(uri!(super::get_pelanggan_by_id(1)))
            .dispatch()
            .await;
        let body = response.into_json::<Pelanggan>().await.unwrap();
        let updated_pelanggan = Pelanggan {
            id: body.id,
            nama: "Aglaea".to_string(),
            alamat: "Okhema".to_string(),
            no_telp: "1234567890".to_string(),
            tanggal_gabung: body.tanggal_gabung,
        };
        let response = client.patch(uri!(super::update_pelanggan(body.id)))
            .json(&updated_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Response>().await.unwrap();
        assert_eq!(body.message, "Pelanggan updated successfully");
    }

    #[async_test]
    async fn test_delete_pelanggan() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let response = client.delete(uri!(super::delete_pelanggan(1)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[async_test]
    async fn test_get_pelanggan_by_id_not_found() {
        let client = setup().await;
        let response = client.get(uri!(super::get_pelanggan_by_id(999)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[async_test]
    async fn test_get_all_pelanggan_sort() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let new_pelanggan_2 = PelangganForm { 
            nama: "Tribbie".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "1234567890".to_string()
        };
        let new_pelanggan_3 = PelangganForm { 
            nama: "Aglaea".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "5432198760".to_string()
        };

        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_2)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_3)
            .dispatch()
            .await;
        let response = client.get(uri!(super::get_all_pelanggan(Some("nama"), _, _)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Pelanggan>>().await.unwrap();
        assert_eq!(body[0].nama, "Aglaea");
        assert_eq!(body[1].nama, "Castorice");
        assert_eq!(body[2].nama, "Tribbie");
    }

    #[async_test]
    async fn test_get_all_pelanggan_filter() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let new_pelanggan_2 = PelangganForm { 
            nama: "Tribbie".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "1234567890".to_string()
        };
        let new_pelanggan_3 = PelangganForm { 
            nama: "Aglaea".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "5432198760".to_string()
        };

        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_2)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_3)
            .dispatch()
            .await;
        let response = client.get(uri!(super::get_all_pelanggan(_, Some("nama"), Some("Agl"))))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Pelanggan>>().await.unwrap();
        assert_eq!(body.len(), 1);
    }

    #[async_test]
    async fn test_get_all_pelanggan_sort_and_filter() {
        let client = setup().await;
        let new_pelanggan = PelangganForm { 
            nama: "Castorice".to_string(), 
            alamat: "Styxia".to_string(), 
            no_telp: "08123456789".to_string()
        };
        let new_pelanggan_2 = PelangganForm { 
            nama: "Tribbie".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "1234567890".to_string()
        };
        let new_pelanggan_3 = PelangganForm { 
            nama: "Aglaea".to_string(), 
            alamat: "Okhema".to_string(), 
            no_telp: "5432198760".to_string()
        };

        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_2)
            .dispatch()
            .await;
        client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan_3)
            .dispatch()
            .await;
        let response = client.get(uri!(super::get_all_pelanggan(Some("nama"), Some("nama"), Some("a"))))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Pelanggan>>().await.unwrap();
        assert_eq!(body[0].nama, "Aglaea");
        assert_eq!(body[1].nama, "Castorice");
    }
}
