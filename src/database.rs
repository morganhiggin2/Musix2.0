use crate::{environment_extractor::EnvironmentVariables, s3_service};
use rusqlite::{self, params};

pub struct Database {
    state: DatabaseState,
}

pub enum DatabaseState {
    UninitializedDatabase(UninitializedDatabase),
    InitializedDatabase(InitializedDatabase),
}

#[derive(Default)]
pub struct UninitializedDatabase {}

pub struct InitializedDatabase {
    connection: rusqlite::Connection,
}

impl Default for Database {
    fn default() -> Self {
        return Database {
            state: DatabaseState::UninitializedDatabase(UninitializedDatabase::default()),
        };
    }
}

impl Database {
    /// Initialize the database if the database has not been initialized
    fn initialize_if_required(
        &mut self,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        match &mut self.state {
            DatabaseState::UninitializedDatabase(state) => {
                let new_state = match InitializedDatabase::new(state, environment_variables) {
                    Ok(some) => DatabaseState::InitializedDatabase(some),
                    Err(e) => return Err(e),
                };

                self.state = new_state;

                return Ok(());
            }
            DatabaseState::InitializedDatabase(_) => return Ok(()),
        };
    }

    fn get_initialized_state_always(
        &mut self,
        environment_variables: &EnvironmentVariables,
    ) -> Result<&mut InitializedDatabase, String> {
        self.initialize_if_required(environment_variables)?;

        return match self.state {
            DatabaseState::InitializedDatabase(ref mut inner_state) => Ok(inner_state),
            _ => Err(String::from("Expected initialized database state")),
        };
    }

    // Wrapper for initiazlied database calls
    pub fn get_downloaded_songs_from_playlist(
        &mut self,
        playlist_url: String,
        environment_variables: &EnvironmentVariables,
    ) -> Result<Vec<String>, String> {
        let initialzied_database = self.get_initialized_state_always(environment_variables)?;

        return initialzied_database.get_downloaded_songs_from_playlist(playlist_url);
    }

    pub fn put_downloaded_song(
        &mut self,
        playlist_url: String,
        song_url: String,
        failed: bool,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        let initialzied_database = self.get_initialized_state_always(environment_variables)?;

        return initialzied_database.put_downloaded_song(playlist_url, song_url, failed);
    }

    pub fn put_playlist(
        &mut self,
        playlist_url: String,
        genre: String,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        let initialzied_database = self.get_initialized_state_always(environment_variables)?;

        return initialzied_database.put_playlist(playlist_url, genre);
    }

    pub fn delete_playlist(
        &mut self,
        playlist_url: String,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        let initialzied_database = self.get_initialized_state_always(environment_variables)?;

        return initialzied_database.delete_playlist(playlist_url);
    }

    pub fn get_all_playlists(
        &mut self,
        environment_variables: &EnvironmentVariables,
    ) -> Result<Vec<String>, String> {
        let initialzied_database = self.get_initialized_state_always(environment_variables)?;

        return initialzied_database.get_all_playlists();
    }
}

impl InitializedDatabase {
    /// Create an InitializedDatabase from a UnintializedDatabase
    pub fn new(
        _: &mut UninitializedDatabase,
        environment_variables: &EnvironmentVariables,
    ) -> Result<InitializedDatabase, String> {
        // Download the database if it does not exist
        // Check if file exists
        match std::fs::exists(std::path::Path::new("data/database/sqlite.db")) {
            Ok(file_exists) => {
                // If the databse does not already exist
                if !file_exists {
                    // Download the database from s3
                    let file_path = std::path::Path::new("data/database/sqlite.db");
                    s3_service::write_s3_object_to_file(
                        environment_variables.get_database_s3_uri().to_owned(),
                        file_path,
                    )?;
                }
            }
            Err(e) => {
                return Err(format!(
                    "File status for data/database/sqlite.db could not be confirmed nor denied: {}",
                    e,
                ))
            }
        };

        //initialize the connection
        let connection = match rusqlite::Connection::open("data/database/sqlite.db") {
            Ok(conn) => conn,
            Err(e) => {
                return Err(format!(
                    "Could not open connection to local sqlite database: {}",
                    e
                ));
            }
        };

        //create / re-establish presence of necessary tables
        let create_table_queries = [
            "CREATE TABLE IF NOT EXISTS playlists (playlist_url VARCHAR(11))",
            "CREATE TABLE IF NOT EXISTS downloaded_songs (song_url VARCHAR(11), playlist_url VARCHAR(11), failed BOOLEAN)",
            "CREATE UNIQUE INDEX IF NOT EXISTS playlists_playlists_id_index ON playlists (playlist_url)",
            "CREATE UNIQUE INDEX IF NOT EXISTS downloaded_songs_song_url ON downloaded_songs (song_url, playlist_id)"
        ];

        //for each create table query
        for create_table_query in create_table_queries.iter() {
            // statement
            let statement_result = connection.execute(create_table_query, params![]);

            match statement_result {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!(
                        "Could not execute create table statement: {}: {}",
                        create_table_query, e
                    ));
                }
            }
        }

        return Ok(InitializedDatabase { connection });
    }

    pub fn get_downloaded_songs_from_playlist(
        &self,
        playlist_url: String,
    ) -> Result<Vec<String>, String> {
        //create query
        let query = "SELECT * FROM downloaded_songs WHERE playlist_url like ?1";

        //list of youtube song ids
        let mut song_urls: Vec<String> = Vec::new();

        //prepare statment
        let mut statement = match self.connection.prepare(query) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not create prepared statement in get downloaded songs : {}: {}",
                    query, e
                ));
            }
        };

        //execute query, map resulting rows
        let songs = match statement.query_map(params![playlist_url], |row| {
            let row_str: String = row.get(0)?;
            Ok(row_str)
        }) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("Could not execute prepared statement and collect row information in get downloaded songs: {}: {}", query, e));
            }
        };

        for song_result in songs {
            let song = match song_result {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!("Error fetching a row for prepared statement {} in get downloaded songs: {}", query, e));
                }
            };

            song_urls.push(song);
        }

        return Ok(song_urls);
    }

    /// Put downloaded song information into database
    ///   If already exists, will silently ignore
    pub fn put_downloaded_song(
        &self,
        playlist_url: String,
        song_url: String,
        failed: bool,
    ) -> Result<(), String> {
        //create query
        let query = "INSERT INTO downloaded_songs VALUES (?1, ?2, ?3) ON CONFLICT(?1) DO NOTHING";

        // execute statement
        let statement_result = self
            .connection
            .execute(&query, params![song_url, playlist_url, failed]);

        //execute query, parse result
        match statement_result {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "Could not execute put downloaded songs query: {}: {}",
                    query, e
                ));
            }
        }

        return Ok(());
    }

    /// Put playlist information into database
    ///   If already exists, will silently ignore
    pub fn put_playlist(&self, playlist_url: String, genre: String) -> Result<(), String> {
        //create query
        let query = "INSERT OR REPLACE INTO playlists(playlist_url, genre) VALUES (?1, ?2)";

        //generate prepared statment
        let _ = match self
            .connection
            .execute(&query, params![playlist_url, genre])
        {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Error in executing perpared statement {}: {}",
                    query, e
                ));
            }
        };

        return Ok(());
    }

    /// Delete playlist from database.
    /// If the playlist does not exist, will do nothing
    pub fn delete_playlist(&self, playlist_url: String) -> Result<(), String> {
        //delete all downloads songs from the downloaded songs table with the playlist id
        let query = "DELETE FROM downloaded_songs WHERE playlist_url = ?1";

        // execute statement
        let _ = match self.connection.execute(&query, params![playlist_url]) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not execute delete downloaded songs of delete playlist query: {}: {}",
                    query, e
                ));
            }
        };

        //delete the playlist from the playlists database
        let query = "DELETE FROM playlists WHERE playlist_url = ?1";

        // execute statement
        let _ = match self.connection.execute(&query, params![playlist_url]) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not execute delete playlist of delete playlist query: {}: {}",
                    query, e
                ));
            }
        };

        return Ok(());
    }

    // Returns a list of tuples containing (playlist id, genre) for each playlist
    pub fn get_all_playlists(&self) -> Result<Vec<String>, String> {
        //create query
        let query = "SELECT * FROM playlists";

        //list of playlist ids
        let mut playlists: Vec<String> = Vec::new();

        //prepare statment
        let mut statement = match self.connection.prepare(query) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not create prepared statement in get all playlists: {}: {}",
                    query, e
                ));
            }
        };

        //execute query, map resulting rows
        let playlists_results = match statement.query_map([], |row| {
            let playlist_url: String = row.get(0)?;

            Ok(playlist_url)
        }) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("Could not execute prepared statement and collect row information in get all playlists: {}: {}", query, e));
            }
        };

        for playlist_result in playlists_results {
            let playlist = match playlist_result {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!("Error fetching a row for prepared statement {} in get downloaded songs: {}", query, e));
                }
            };

            playlists.push(playlist);
        }

        return Ok(playlists);
    }
}
