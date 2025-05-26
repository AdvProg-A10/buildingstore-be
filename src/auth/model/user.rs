use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

impl User {
    pub fn new(username: String, password: String, is_admin: bool) -> Self {
        User {
            id: 0,
            username,
            password: bcrypt::hash(password, bcrypt::DEFAULT_COST).expect("Failed to hash password"),
            is_admin,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password).unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_default_user() {
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        assert_eq!(user.username, "test_user");
        assert_eq!(user.is_admin, false);
    }

    #[test]
    fn test_create_admin_user() {
        let user = User::new("test_user".to_string(), "password".to_string(), true);
        assert_eq!(user.username, "test_user");
        assert_eq!(user.is_admin, true);
    }

    #[test]
    fn test_verify_correct_password() {
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        assert!(user.verify_password("password"));
    }

    #[test]
    fn test_verify_wrong_password() {
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        assert!(!user.verify_password("wrong_password"));
    }
}