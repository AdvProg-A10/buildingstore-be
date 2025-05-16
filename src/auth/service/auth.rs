use sqlx::{Any, Pool};
use uuid::Uuid;

use crate::auth::model::user::User;
use crate::auth::model::session::Session;
use crate::auth::repository::user::UserRepository;
use crate::auth::repository::session::SessionRepository;

pub struct AuthService;

impl AuthService {
    pub async fn register_user(db: Pool<Any>, user: User) -> Result<User, sqlx::Error> {
        let existing_user = UserRepository::get_user_by_username(db.acquire().await.unwrap(), &user.username).await;
        if existing_user.is_ok() {
            return Err(sqlx::Error::RowNotFound);
        }
        let new_user = UserRepository::create_user(db.acquire().await.unwrap(), user).await?;
        Ok(new_user)
    }

    pub async fn login_user(db: Pool<Any>, username: String, password: String) -> Result<Session, sqlx::Error> {
        let existing_user = UserRepository::get_user_by_username(db.acquire().await.unwrap(), &username).await;
        if existing_user.is_err() {
            return Err(sqlx::Error::RowNotFound);
        }
        let existing_user = existing_user.unwrap();
        let is_password_valid = existing_user.verify_password(&password);
        if !is_password_valid {
            return Err(sqlx::Error::RowNotFound);
        }
        let session = Session::new(existing_user.clone());
        SessionRepository::create_session(db.acquire().await.unwrap(), session.clone()).await?;
        Ok(session)
    }

    pub async fn logout_user(db: Pool<Any>, session_key: Uuid) -> Result<(), sqlx::Error> {
        SessionRepository::delete_session(db.acquire().await.unwrap(), session_key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sqlx::any::install_default_drivers;
    use sqlx::Pool;
    use crate::auth::model::user::User;

    async fn setup() -> Pool<Any> {
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

        db
    }

    #[async_test]
    async fn test_register_user() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let result = AuthService::register_user(db, user.clone()).await;
        assert!(result.is_ok());

        let registered_user = result.unwrap();
        assert_eq!(registered_user.username, user.username);
        assert_eq!(registered_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_login_valid_user_credentials() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let mut user = User::new(username.clone(), password.clone(), false);

        user = AuthService::register_user(db.clone(), user.clone()).await.unwrap();
        let result = AuthService::login_user(db.clone(), username.clone(), password.clone()).await;
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.user_id, user.id);
    }

    #[async_test]
    async fn test_login_invalid_username() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let user = User::new(username.clone(), password.clone(), false);

        AuthService::register_user(db.clone(), user.clone()).await.unwrap();

        let result = AuthService::login_user(db.clone(), "dummy".to_string(), password.clone()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_login_invalid_password() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let user = User::new(username.clone(), password.clone(), false);

        AuthService::register_user(db.clone(), user.clone()).await.unwrap();

        let result = AuthService::login_user(db.clone(), username.clone(), "dummypass".to_string()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_logout_user() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let user = User::new(username.clone(), password.clone(), false);

        AuthService::register_user(db.clone(), user.clone()).await.unwrap();
        let session = AuthService::login_user(db.clone(), username.clone(), password.clone()).await.unwrap();
        let result = AuthService::logout_user(db.clone(), Uuid::try_parse(&session.session_key).unwrap()).await;
        assert!(result.is_ok());
    }
}