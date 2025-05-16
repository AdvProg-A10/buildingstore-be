use rocket_db_pools::sqlx;
use sqlx::{Any, Row};
use sqlx::pool::PoolConnection;
use crate::auth::model::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn create_user(mut db: PoolConnection<Any>, user: User) -> Result<User, sqlx::Error> {
        let row = sqlx::query("INSERT INTO users (username, password, is_admin) VALUES ($1, $2, $3) RETURNING id")
            .bind(&user.username)
            .bind(&user.password)
            .bind(user.is_admin as i32)
            .fetch_one(&mut *db)
            .await?;

        let id: i64 = row.get("id");
        Ok(User {
            id,
            username: user.username,
            password: user.password,
            is_admin: user.is_admin,
        })
    }

    pub async fn get_user_by_username(mut db: PoolConnection<Any>, username: &str) -> Result<User, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&mut *db)
            .await?;

        let id: i64 = row.get("id");
        let password: String = row.get("password");
        let is_admin: bool = match row.try_get("is_admin") {
            Ok(value) => value,
            Err(_) => {
                let is_admin_int: i32 = row.get("is_admin");
                is_admin_int != 0
            }
        };

        Ok(User {
            id,
            username: username.to_string(),
            password,
            is_admin,
        })
    }

    pub async fn get_user_by_id(mut db: PoolConnection<Any>, user_id: i64) -> Result<User, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *db)
            .await?;

        let username: String = row.get("username");
        let password: String = row.get("password");
        let is_admin: bool = match row.try_get("is_admin") {
            Ok(value) => value,
            Err(_) => {
                let is_admin_int: i32 = row.get("is_admin");
                is_admin_int != 0
            }
        };

        Ok(User {
            id: user_id,
            username,
            password,
            is_admin,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sqlx::any::install_default_drivers;
    use sqlx::Pool;

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
    async fn test_create_user() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let result = UserRepository::create_user(db.acquire().await.unwrap(), user.clone()).await;
        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.username, user.username);
        assert_eq!(created_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_username() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        UserRepository::create_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let result = UserRepository::get_user_by_username(db.acquire().await.unwrap(), &user.username).await;
        assert!(result.is_ok());
        let fetched_user = result.unwrap();
        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_nonexistent_username() {
        let db = setup().await;
        let result = UserRepository::get_user_by_username(db.acquire().await.unwrap(), "nonexistent_user").await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_get_user_by_id() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let created_user = UserRepository::create_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let result = UserRepository::get_user_by_id(db.acquire().await.unwrap(), created_user.id).await;
        assert!(result.is_ok());
        let fetched_user = result.unwrap();
        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_nonexistent_id() {
        let db = setup().await;
        let result = UserRepository::get_user_by_id(db.acquire().await.unwrap(), 999).await;
        assert!(result.is_err());
    }
}