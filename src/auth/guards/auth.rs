use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use sqlx::{Any, Pool};
use uuid::Uuid;

use crate::auth::repository::session::SessionRepository;
use crate::auth::repository::user::UserRepository;

pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
    pub is_admin: bool,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        let db = request.guard::<&State<Pool<Any>>>().await.unwrap();
        let session_key = cookies.get_private("session_key").map(|c| c.value().to_string());
        if session_key.is_none() {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        let session_key = session_key.unwrap();
        let session = SessionRepository::get_session_by_key(db.acquire().await.unwrap(), Uuid::try_parse(&session_key).unwrap()).await;
        match session {
            Ok(session) => 
            {
                if !session.is_valid() {
                    return Outcome::Error((Status::Unauthorized, ()));
                }
                let user = UserRepository::get_user_by_id(db.acquire().await.unwrap(), session.user_id).await.unwrap();
                return Outcome::Success(AuthenticatedUser {
                    user_id: user.id,
                    username: user.username,
                    is_admin: user.is_admin,
                });
            },
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };
    }
}