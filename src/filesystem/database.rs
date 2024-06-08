use std::fs::File;
use std::io::Write;
use std::path::Path;

use sqlite::State;
use uuid::Uuid;

use crate::api::location::Location;
use crate::api::user::User;
use crate::logging;

/// Create a folder for the user, and initialize the database where the user data will be stored.
pub fn initialize_new_user(user: &User) {
    create_user_dir(user.uuid.to_string().as_str());
    let filename = format!("data/db/users/{}/location_data.db", &user.uuid);
    let connection = sqlite::open(filename).unwrap();
    let query = "CREATE TABLE location (latitude INTEGER, longitude INTEGER, gathered_at INTEGER);";

    connection.execute(query).unwrap();
    logging::info(format!("Created database for user {}. (device name: {})", &user.uuid, &user.device_name), Some("database"));
    add_user_to_users_db(user);
    create_user_gpx(user.uuid);
}

/// Create user directories
fn create_user_dir(name: &str) {
    let path_string = format!("./data/db/users/{}", name);
    let path = Path::new(&path_string);
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }

    let path_string = format!("./data/gpx/users/{}", name);
    let path = Path::new(&path_string);
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

/// Create empty gpx file
fn create_user_gpx(uuid: Uuid) {
    let file_location = format!("./data/gpx/users/{}/location_data.gpx", uuid);
    let mut file = File::create(&file_location).expect("create gpx error.");
    
    // write empty gpx track etc.
    file.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gpx\n\tversion=\"1.1\"\n\tcreator=\"gpslog\">\n\n<trk>\n\t<name>gpslog</name>\n\t<trkseg>\n\t</trkseg>\n</gpx>").unwrap();
    logging::info(format!("Created GPX file for user {}", uuid), Some("database"));
}


/// Append the user to the users database. If the database does not exist, create it.
fn add_user_to_users_db(user: &User) {
    let file = Path::new("./data/db/users/users.db");
    let query: String;

    // If the database does not exist, modify the query to also create the table.
    if !file.exists() {
        // Writes header and user.
        query = format!("CREATE TABLE users (name TEXT, device_name TEXT, created_at INTEGER); INSERT INTO users (name, device_name, created_at) VALUES ('{}','{}', {});", user.uuid, user.device_name, user.created_at.timestamp());
    } else {
        // Only writes user.
        query = format!("INSERT INTO users (name, device_name, created_at) VALUES ('{}','{}', {});", user.uuid, user.device_name, user.created_at.timestamp());
    }

    let connection = sqlite::open("data/db/users/users.db").unwrap();
    connection.execute(query).unwrap();
}


/// Adds a [`Location`] to a user's database.
pub fn add_location_to_user_db(data: Location) {
    let db_file = format!("data/db/users/{}/location_data.db", data.get_uuid());
    let connection = sqlite::open(db_file).unwrap();
    connection.execute(format!("INSERT INTO location (latitude, longitude, gathered_at) VALUES ({}, {}, {})", data.get_lat_long().0, data.get_lat_long().1, data.get_gathered_at())).unwrap()
}

// TODO: Learn more about lifetimes!
/// Fetches the list of users from the database.
///
/// # Returns
/// Either
/// * [`Ok`] With a [`Vec<String>`] with a list of user UUIDs.
/// * [`Err`] With an error message.
pub fn fetch_users() -> Result<Vec<String>, &'static str> {
    let users_db = Path::new("./data/db/users/users.db");
    let mut users: Vec<String> = Vec::new();
    // Check if the file even exists
    if !users_db.exists() {
        return Err("The users database file was not found. Perhaps there are no users registered yet?");
    }

    let query = "SELECT name FROM users";
    let connection = sqlite::open("data/db/users/users.db").unwrap();

    let mut statement = connection.prepare(query).unwrap();

    while let Ok(State::Row) = statement.next() {
        let uuid = statement.read::<String, _>("name").unwrap();
        users.push(uuid);
    }

    Ok(users)
}