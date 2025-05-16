use rocket_db_pools::sqlx;
use sqlx::{Any, Row};
use sqlx::pool::PoolConnection;
use uuid::Uuid;
use chrono::DateTime;
use crate::auth::model::session::Session;

pub struct SessionRepository;

impl SessionRepository {
    pub async fn create_session(mut db: PoolConnection<Any>, session: Session) -> Result<Session, sqlx::Error> {
        sqlx::query("INSERT INTO sessions (session_key, user_id, expires_at) VALUES ($1, $2, $3)")
            .bind(&session.session_key)
            .bind(&session.user_id)
            .bind(&session.expires_at.to_rfc3339())
            .execute(&mut *db)
            .await?;

        Ok(session)
    }

    pub async fn get_session_by_key(mut db: PoolConnection<Any>, session_key: Uuid) -> Result<Session, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM sessions WHERE session_key = $1")
            .bind(session_key.to_string())
            .fetch_one(&mut *db)
            .await?;

        let session_key: String = row.get("session_key");
        let user_id: i64 = row.get("user_id");
        let expires_at: String = row.get("expires_at");

        Ok(Session {
            session_key,
            user_id,
            expires_at: DateTime::parse_from_rfc3339(&expires_at)
                .expect("Failed to parse expires_at")
                .with_timezone(&chrono::Utc),
        })
    }

    pub async fn delete_session(mut db: PoolConnection<Any>, session_key: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sessions WHERE session_key = $1")
            .bind(session_key.to_string())
            .execute(&mut *db)
            .await?;

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
    use crate::auth::repository::user::UserRepository;

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
    async fn test_create_session() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let session = Session::new(user.clone());
        let result = SessionRepository::create_session(db.acquire().await.unwrap(), session.clone()).await;
        assert!(result.is_ok());
        let fetched_session = result.unwrap();
        assert_eq!(fetched_session.session_key, session.session_key);
        assert_eq!(fetched_session.user_id, session.user_id);
        assert_eq!(fetched_session.expires_at, session.expires_at);
    }

    #[async_test]
    async fn test_get_session_by_key() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let session = Session::new(user.clone());
        SessionRepository::create_session(db.acquire().await.unwrap(), session.clone()).await.unwrap();
        let retrieved_session = SessionRepository::get_session_by_key(db.acquire().await.unwrap(), Uuid::parse_str(&session.session_key).unwrap()).await;
        assert!(retrieved_session.is_ok());
        let fetched_session = retrieved_session.unwrap();
        assert_eq!(fetched_session.session_key, session.session_key);
        assert_eq!(fetched_session.user_id, session.user_id);
        assert_eq!(fetched_session.expires_at, session.expires_at);
    }

    #[async_test]
    async fn test_get_session_by_nonexistent_key() {
        let db = setup().await;
        let result = UserRepository::get_user_by_username(db.acquire().await.unwrap(), "nonexistent_user").await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_delete_session() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let session = Session::new(user.clone());
        SessionRepository::create_session(db.acquire().await.unwrap(), session.clone()).await.unwrap();
        let result = SessionRepository::delete_session(db.acquire().await.unwrap(), Uuid::parse_str(&session.session_key).unwrap()).await;
        assert!(result.is_ok());
        let retrieved_session = SessionRepository::get_session_by_key(db.acquire().await.unwrap(), Uuid::parse_str(&session.session_key).unwrap()).await;
        assert!(retrieved_session.is_err());
    }
}