use rocket::serde::json::Json;
use rocket::{get, post, patch, State};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::{Deserialize, Serialize};
use sqlx::{Any, Pool};
use uuid::Uuid;

use crate::auth::model::user::User;
use crate::auth::service::auth::AuthService;
use crate::auth::guards::auth::AuthenticatedUser;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ChangePasswordForm {
    pub new_password: String,
}

#[get("/user")]
pub async fn get_user(user: AuthenticatedUser) -> Json<AuthenticatedUser> {
    Json(user)
}

#[post("/login", data = "<form>")]
pub async fn login(form: Json<AuthForm>, cookies: &CookieJar<'_>, db: &State<Pool<Any>>, production: &State<bool>) -> Status {
    let username = form.username.clone();
    let password = form.password.clone();

    let result = AuthService::login_user(db.inner().clone(), username, password).await;
    match result {
        Ok(session) => {
            let mut cookie = Cookie::new("session_key", session.session_key);
            cookie.set_same_site(SameSite::None);
            cookie.set_partitioned(true);
            if *production.inner() {
                cookie.set_domain("koyeb.app");
            }
            cookies.add_private(cookie);
            Status::Ok
        },
        Err(_) => Status::Unauthorized,
    }
}

#[post("/register", data = "<form>")]
pub async fn register(user: AuthenticatedUser, form: Json<RegisterForm>, db: &State<Pool<Any>>) -> Status {
    if !user.is_admin {
        return Status::Unauthorized;
    }
    let username = form.username.clone();
    let password = form.password.clone();
    let is_admin = form.is_admin;

    let user = User::new(username, password, is_admin);
    let result = AuthService::register_user(db.inner().clone(), user).await;
    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::BadRequest
    }
}

#[get("/logout")]
pub async fn logout(db: &State<Pool<Any>>, cookies: &CookieJar<'_>) -> Status {
    let session_key = cookies.get_private("session_key").map(|c| c.value().to_string()).unwrap_or_default();
    if session_key.is_empty() {
        return Status::BadRequest;
    }
    AuthService::logout_user(db.inner().clone(), Uuid::try_parse(&session_key).unwrap()).await.unwrap();
    cookies.remove_private(Cookie::build("session_key"));
    Status::Ok
}

#[patch("/change_password", data = "<form>")]
pub async fn change_password(user: AuthenticatedUser, form: Json<ChangePasswordForm>, db: &State<Pool<Any>>) -> Status {
    let new_password = form.new_password.clone();

    let result = AuthService::update_user_password(db.inner().clone(), user.user_id, new_password).await;
    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::Unauthorized
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::http::Status;
    use rocket::{routes, uri, Rocket, async_test};
    use sqlx::any::install_default_drivers;

    const ADMIN_USERNAME: &str = "admin";
    const ADMIN_PASSWORD: &str = "adminpass";

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

        let admin_user = User::new(ADMIN_USERNAME.to_string(), ADMIN_PASSWORD.to_string(), true);
        AuthService::register_user(db.clone(), admin_user).await.unwrap();

        let production = false;

        let rocket = rocket::build()
            .manage(reqwest::Client::builder().build().unwrap())
            .manage(db.clone())
            .manage(production)
            .mount("/", routes![login, register, logout, change_password, get_user]);

        rocket
    }

    #[async_test]
    async fn test_login_valid_credentials() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let response = client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"{}","password":"{}"}}"#, ADMIN_USERNAME, ADMIN_PASSWORD))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let cookies = response.cookies();
        assert!(cookies.get("session_key").is_some(), "Session cookie should be set");
    }

    #[async_test]
    async fn test_login_invalid_credentials() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let response = client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"invaliduser","password":"invalidpass"}}"#))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Unauthorized);
        let cookies = response.cookies();
        assert!(cookies.get("session_key").is_none(), "Session cookie should not be set");
    }

    #[async_test]
    async fn test_get_user() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"{}","password":"{}"}}"#, ADMIN_USERNAME, ADMIN_PASSWORD))
            .dispatch()
            .await;
        let response = client.get(uri!(super::get_user))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[async_test]
    async fn test_get_user_unauthorized() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let response = client.get(uri!(super::get_user))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[async_test]
    async fn test_register() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"{}","password":"{}"}}"#, ADMIN_USERNAME, ADMIN_PASSWORD))
            .dispatch()
            .await;
        let response = client.post(uri!(super::register))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"test","password":"testuser","is_admin":false}}"#))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[async_test]
    async fn test_register_existing_user() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"{}","password":"{}"}}"#, ADMIN_USERNAME, ADMIN_PASSWORD))
            .dispatch()
            .await;
        client.post(uri!(super::register))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"testuser","password":"testpass","is_admin":false}}"#))
            .dispatch()
            .await;
        let response = client.post(uri!(super::register))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"testuser","password":"testpass","is_admin":false}}"#))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[async_test]
    async fn test_register_not_admin() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let response = client.post(uri!(super::register))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"testuser","password":"testpass","is_admin":false}}"#))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[async_test]
    async fn test_logout() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"{}","password":"{}"}}"#, ADMIN_USERNAME, ADMIN_PASSWORD))
            .dispatch()
            .await;
        let response = client.get(uri!(super::logout))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let cookies = response.cookies();
        let session_key = cookies.get("session_key").map(|c| c.value()).unwrap_or_default();
        assert!(session_key.is_empty(), "Session cookie should be cleared");
    }

    #[async_test]
    async fn test_change_password() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::login))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"username":"{}","password":"{}"}}"#, ADMIN_USERNAME, ADMIN_PASSWORD))
            .dispatch()
            .await;
        let response = client.patch(uri!(super::change_password))
            .header(rocket::http::ContentType::JSON)
            .body(format!(r#"{{"new_password":"newpassword"}}"#))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }
}