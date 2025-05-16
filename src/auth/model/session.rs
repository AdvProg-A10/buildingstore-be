use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::auth::model::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Session {
    pub session_key: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user: User) -> Self {
        Session {
            session_key: Uuid::new_v4().to_string(),
            user_id: user.id,
            expires_at: Utc::now() + chrono::Duration::hours(24),
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn generate_session_key() -> String {
        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_session() {
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        let session = Session::new(user.clone());
        assert_eq!(session.user_id, 0);
        assert_eq!(session.session_key.len(), 36);
    }

    #[test]
    fn test_session_is_valid() {
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        let session = Session::new(user.clone());
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_is_not_valid() {
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        let mut session = Session::new(user.clone());
        session.expires_at = Utc::now() - chrono::Duration::hours(1);
        assert!(!session.is_valid());
    }

    #[test]
    fn test_generate_session_key() {
        let session_key = Session::generate_session_key();
        assert_eq!(session_key.len(), 36);
    }
}