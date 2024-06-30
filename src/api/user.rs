use std::fmt::Debug;

use actix_web::{get, HttpRequest, HttpResponse, post, Responder, web};
use actix_web::web::Json;
use chrono;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::filesystem::database::{fetch_users, get_user_from_users_db, initialize_new_user};
use crate::logging::{error, info};

/// Represents a user in the system.
#[derive(Debug)]
pub struct User {
    pub uuid: String,
    pub device_name: String,
    pub created_at: DateTime<Utc>,
}

/// Represents the request body for creating a new user. The json that is received should have a device_id field.  
/// This struct is used to deserialize the request body into a UserRequest struct and then create a new [`User`].
#[derive(Debug, Deserialize, Serialize)]
pub struct UserRequest {
    pub device_id: String,
}


impl User {
    /// Create a new [`User`] with a random UUIDv4 and the current time.
    pub fn new(device_identifier: String) -> User {
        User {
            uuid: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            device_name: device_identifier,
        }
    }

    /// Creates a [`User`] from a UUID. Make sure the user exists in the users DB.
    pub fn from_uuid(uuid: String) -> Result<User, &'static str> {
        match get_user_from_users_db(uuid.clone()) {
            Ok((_, device_name, created_at)) => {
                // println!("UUID: {} | DEVICE NAME: {} | CREATED_AT: {}",uuid, &device_name, &created_at);
                Ok(User {
                    uuid,
                    created_at: DateTime::from_timestamp(created_at, 0).unwrap(),
                    device_name,
                })
            }
            Err(msg) => {
                Err(msg)
            }
        }
    }
}

/// Create a new [`User`] and return the generated UUIDv4.
#[post("/api/users")]
pub async fn create_user(req_body: Json<UserRequest>, request: HttpRequest) -> impl Responder {
    if req_body.device_id.contains('"') || req_body.device_id.contains('\'') { return HttpResponse::BadRequest().body("Device ID can't contain special characters (', \")."); } // TODO: Handle special characters in device IDs

    let new_user = User::new(req_body.into_inner().device_id);

    initialize_new_user(&new_user); // Create database.
    info(format!("IP {}: Created new user {}.", request.peer_addr().unwrap().ip(), &new_user.uuid), Some("POST: /api/users"));

    HttpResponse::Created().body(new_user.uuid)
}

/// Get a list of all users in the system.
/// # Returns
/// A JSON array with all the users in the system.
/// # Errors
/// If the lookup of users fails, an internal server error is returned.
#[get("/api/users")]
pub async fn get_users(request: HttpRequest) -> HttpResponse {
    // We build a shitty JSON array by hand. This is fine for now but not future-proof.
    let mut response = String::from("[");

    let users = match fetch_users() {
        Ok(users) => users,
        Err(e) => {
            error(format!("IP {} requested list of users, but the lookup failed with error: {e}", request.peer_addr().unwrap().ip()), Some("list_users"));
            return HttpResponse::InternalServerError().body("Could not fetch users. Please try again later.");
        }
    };

    for user_uuid in users.iter() {
        // println!("User {:?}", user_uuid);
        response.push_str(format!("\"{}\",", user_uuid).as_str());
    }

    response.pop(); // Remove the last comma.
    response.push(']');

    info(format!("IP {}: Requested list of users.", request.peer_addr().unwrap().ip()), Some("GET: /api/users"));
    HttpResponse::Ok().body(response)
}


/// First converts a user's UUID into a [`User`], and returns its info.
/// If the user is not found in the users DB, return an [`InternalServerError`].
#[get("/api/users/{uuid}")]
pub async fn get_user(path: web::Path<String>, request: HttpRequest) -> HttpResponse {
    let requested_uuid = path.into_inner().to_string();
    match User::from_uuid(requested_uuid.clone()) {
        Ok(user) => {
            info(format!("IP {} requested user lookup for user {}", request.peer_addr().unwrap().ip(), user.uuid), Some("get_user"));
            HttpResponse::Ok().body(format!("UUID: {}\nDEVICE NAME: {}\nCREATED AT: {}", user.uuid, user.device_name, user.created_at))
        }
        Err(msg) => {
            error(format!("IP {} requested user lookup for (possibly non-existent) user {}. Lookup failed with error: {}", request.peer_addr().unwrap().ip(), requested_uuid, msg), Some("get_user"));

            match msg {
                "User not in users DB" => {
                    HttpResponse::InternalServerError().body("User not found on the server.")
                }
                "Users DB doesn't exist!" => {
                    HttpResponse::InternalServerError().body("There are no users on this server yet!")
                }
                _ => {
                    HttpResponse::InternalServerError().body(msg)
                }
            }
        }
    }
}