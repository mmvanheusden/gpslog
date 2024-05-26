use std::fmt::Debug;

use actix_web::{get, HttpResponse, post, Responder};
use actix_web::web::{Json, Path};
use chrono;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::logging::info;


/// The [`User`] struct represents a user in the system.
#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub device_id: String,
    pub created_at: DateTime<Utc>,
}

/// The [`UserRequest`] struct represents the request body for creating a new user. The json that is received should have a device_id field.  
/// This struct is used to deserialize the request body into a UserRequest struct and then create a new [`User`].
#[derive(Debug, Deserialize, Serialize)]
pub struct UserRequest {
    pub device_id: String,
}


impl User {
    /// Create a new [`User`] with a random UUIDv4 and the current time.
    pub fn new(device_identifier: String) -> User {
        User {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            device_id: device_identifier,
        }
    }
}

/// Create a new [`User`] and return the generated UUIDv4.
#[post("/create_user")]
pub async fn create_user(req_body: Json<UserRequest>) -> impl Responder {
    let new_user = User::new(req_body.into_inner().device_id);
    HttpResponse::Created().body(new_user.id.to_string())
}

#[get("/get_user/{id}")]
pub async fn get_user(path: Path<(Uuid, )>) -> HttpResponse {
    //TODO
    info("taco", Some("get_user"));
    HttpResponse::NotImplemented().body("<h1>Not implemented.</h1>")
}
