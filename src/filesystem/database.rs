use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time;

use sqlite::State;
use uuid::Uuid;

use crate::api::geospatial::Location;
use crate::api::user::User;
use crate::logging;

/// Create a folder for the user, and initialize the database where the user data will be stored.
pub fn initialize_new_user(user: &User) {
    init_user_filestructure(&user);
    add_user_to_users_db(user);
    logging::info(format!("Created database for user {}. (device name: {})", &user.uuid, &user.device_name), Some("database"));
}

/**
 * * Creates a directory structure for the specified [`User`]
 * * Creates empty database,gpx file for user.
 **/
fn init_user_filestructure(user: &User) {
    // DIRECTORIES
    // db
    let path_string = format!("./data/db/users/{}", &user.uuid);
    let path = Path::new(&path_string);
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }

    // gpx
    let path_string = format!("./data/gpx/users/{}", &user.uuid);
    let path = Path::new(&path_string);
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }


    // FILES
    // Create user empty gpx
    let file_location = format!("./data/gpx/users/{}/location_data.gpx", &user.uuid);
    let mut file = File::create(&file_location).expect("create gpx error.");

    // write empty gpx track
    file.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gpx\n\tversion=\"1.1\"\n\tcreator=\"gpslog\">\n\n<trk>\n\t<name>gpslog</name>\n\t<trkseg>\n\t</trkseg>\n</gpx>").unwrap();
    logging::info(format!("Created GPX file for user {}", &user.uuid), Some("database"));

    // Create user db
    let filename = format!("data/db/users/{}/location_data.db", &user.uuid);
    let connection = sqlite::open(filename).unwrap();
    let query = "CREATE TABLE location (latitude INTEGER, longitude INTEGER, gathered_at INTEGER);";

    connection.execute(query).unwrap();
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
pub fn add_location_to_user_db(uuid: &Uuid, location: &Location) {
    let db_file = format!("data/db/users/{}/location_data.db", &uuid);
    let connection = sqlite::open(db_file).unwrap();
    connection.execute(format!("INSERT INTO location (latitude, longitude, gathered_at) VALUES ({}, {}, {})", location.lat(), location.lon(), time::UNIX_EPOCH.elapsed().unwrap().as_millis())).unwrap()
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