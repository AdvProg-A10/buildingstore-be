use rocket::{get, post, patch, delete};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::{Any, Pool};

use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;
use crate::manajemen_pelanggan::service::pelanggan::PelangganService;

#[get("/pelanggan?<sort>&<filter>&<keyword>")]
pub async fn get_all_pelanggan(db: &State<Pool<Any>>, sort: Option<String>, filter: Option<String>, keyword: Option<String>) -> Result<Json<Vec<Pelanggan>>, Status> {
    let pelanggan = PelangganService::get_all_pelanggan(db.inner().clone()).await.map_err(|_| Status::InternalServerError)?;
    let mut pelanggan = pelanggan.clone();
    if let Some(sort_strategy) = &sort {
        pelanggan = PelangganService::sort_pelanggan(pelanggan, sort_strategy);
    }
    if let Some(filter_strategy) = &filter {
        if let Some(keyword_value) = &keyword {
            pelanggan = PelangganService::filter_pelanggan(pelanggan, filter_strategy, keyword_value);
        }
    }
    Ok(Json(pelanggan))
}

#[post("/pelanggan", data = "<pelanggan>")]
pub async fn create_pelanggan(db: &State<Pool<Any>>, pelanggan: Json<Pelanggan>) -> Result<Json<Pelanggan>, Status> {
    let new_pelanggan = PelangganService::create_pelanggan(db.inner().clone(), &pelanggan).await.map_err(|_| Status::InternalServerError)?;
    Ok(Json(new_pelanggan))
}

#[get("/pelanggan/<id>")]
pub async fn get_pelanggan_by_id(db: &State<Pool<Any>>, id: i32) -> Result<Json<Pelanggan>, Status> {
    let pelanggan = PelangganService::get_pelanggan_by_id(db.inner().clone(), id).await.map_err(|_| Status::NotFound)?;
    Ok(Json(pelanggan))
}

#[patch("/pelanggan/<id>", data = "<pelanggan>")]
pub async fn update_pelanggan(db: &State<Pool<Any>>, id: i32, pelanggan: Json<Pelanggan>) -> Result<Json<Pelanggan>, Status> {
    if pelanggan.id != id {
        return Err(Status::BadRequest);
    }
    let updated_pelanggan = PelangganService::update_pelanggan(db.inner().clone(), &pelanggan).await.map_err(|_| Status::InternalServerError)?;
    Ok(Json(updated_pelanggan))
}

#[delete("/pelanggan/<id>")]
pub async fn delete_pelanggan(db: &State<Pool<Any>>, id: i32) -> Result<Status, Status> {
    PelangganService::delete_pelanggan(db.inner().clone(), id).await.map_err(|_| Status::InternalServerError)?;
    Ok(Status::NoContent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{routes, uri, Rocket, async_test};
    use sqlx::any::install_default_drivers;
    use crate::manajemen_pelanggan::model::pelanggan::Pelanggan;

    async fn setup() -> Rocket<rocket::Build> {
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

        let rocket = rocket::build()
            .manage(db.clone())
            .mount("/", routes![get_all_pelanggan, create_pelanggan, 
            get_pelanggan_by_id, update_pelanggan, delete_pelanggan]);

        rocket
    }

    #[async_test]
    async fn test_create_pelanggan() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Pelanggan>().await.unwrap();
        assert_eq!(body.nama, new_pelanggan.nama);
        assert_eq!(body.alamat, new_pelanggan.alamat);
    }


    #[async_test]
    async fn test_get_all_pelanggan() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let new_pelanggan_2 = Pelanggan::new("Tribbie".to_string(), "Okhema".to_string(), "1234567890".to_string());

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
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Pelanggan>().await.unwrap();
        let response = client.get(uri!(super::get_pelanggan_by_id(body.id)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Pelanggan>().await.unwrap();
        assert_eq!(body.nama, new_pelanggan.nama);
    }

    #[async_test]
    async fn test_update_pelanggan() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
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
        let body = response.into_json::<Pelanggan>().await.unwrap();
        assert_eq!(body.nama, updated_pelanggan.nama);
    }

    #[async_test]
    async fn test_delete_pelanggan() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let response = client.post(uri!(super::create_pelanggan))
            .json(&new_pelanggan)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Pelanggan>().await.unwrap();
        let response = client.delete(uri!(super::delete_pelanggan(body.id)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::NoContent);
    }

    #[async_test]
    async fn test_get_pelanggan_by_id_not_found() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let response = client.get(uri!(super::get_pelanggan_by_id(999)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[async_test]
    async fn test_get_all_pelanggan_sort() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let new_pelanggan_2 = Pelanggan::new("Tribbie".to_string(), "Okhema".to_string(), "1234567890".to_string());
        let new_pelanggan_3 = Pelanggan::new("Aglaea".to_string(), "Okhema".to_string(), "5432198760".to_string());

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
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let new_pelanggan_2 = Pelanggan::new("Tribbie".to_string(), "Okhema".to_string(), "1234567890".to_string());
        let new_pelanggan_3 = Pelanggan::new("Aglaea".to_string(), "Okhema".to_string(), "5432198760".to_string());

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
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let new_pelanggan = Pelanggan::new("Castorice".to_string(), "Styxia".to_string(), "08123456789".to_string());
        let new_pelanggan_2 = Pelanggan::new("Tribbie".to_string(), "Okhema".to_string(), "1234567890".to_string());
        let new_pelanggan_3 = Pelanggan::new("Aglaea".to_string(), "Okhema".to_string(), "5432198760".to_string());

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
