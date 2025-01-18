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
    async fn initialize_if_required(
        &mut self,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        match &mut self.state {
            DatabaseState::UninitializedDatabase(state) => {
                let new_state = match InitializedDatabase::new(state, environment_variables).await {
                    Ok(some) => DatabaseState::InitializedDatabase(some),
                    Err(e) => return Err(e),
                };

                self.state = new_state;

                return Ok(());
            }
            DatabaseState::InitializedDatabase(_) => return Ok(()),
        };
    }

    async fn get_initialized_state_always(
        &mut self,
        environment_variables: &EnvironmentVariables,
    ) -> Result<&mut InitializedDatabase, String> {
        self.initialize_if_required(environment_variables).await?;

        return match self.state {
            DatabaseState::InitializedDatabase(ref mut inner_state) => Ok(inner_state),
            _ => Err(String::from("Expected initialized database state")),
        };
    }

    // Wrapper for initiazlied database calls
    pub async fn get_downloaded_videos_from_playlist(
        &mut self,
        playlist_id: String,
        environment_variables: &EnvironmentVariables,
    ) -> Result<Vec<String>, String> {
        let initialzied_database = self
            .get_initialized_state_always(environment_variables)
            .await?;

        return initialzied_database.get_downloaded_videos_from_playlist(playlist_id);
    }

    pub async fn put_downloaded_video(
        &mut self,
        playlist_id: String,
        video_id: String,
        failed: bool,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        let initialzied_database = self
            .get_initialized_state_always(environment_variables)
            .await?;

        return initialzied_database.put_downloaded_video(playlist_id, video_id, failed);
    }

    pub async fn put_playlist(
        &mut self,
        playlist_id: String,
        genre: String,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        let initialzied_database = self
            .get_initialized_state_always(environment_variables)
            .await?;

        return initialzied_database.put_playlist(playlist_id, genre);
    }

    pub async fn delete_playlist(
        &mut self,
        playlist_id: String,
        environment_variables: &EnvironmentVariables,
    ) -> Result<(), String> {
        let initialzied_database = self
            .get_initialized_state_always(environment_variables)
            .await?;

        return initialzied_database.delete_playlist(playlist_id);
    }

    pub async fn get_all_playlists(
        &mut self,
        environment_variables: &EnvironmentVariables,
    ) -> Result<Vec<(String, String)>, String> {
        let initialzied_database = self
            .get_initialized_state_always(environment_variables)
            .await?;

        return initialzied_database.get_all_playlists();
    }
}

impl InitializedDatabase {
    /// Create an InitializedDatabase from a UnintializedDatabase
    pub async fn new(
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
                    )
                    .await?;
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
        let connection = rusqlite::Connection::open("data/database/sqlite.db").unwrap();

        //create / re-establish presence of necessary tables
        let create_table_queries = [
            "CREATE TABLE IF NOT EXISTS playlists (playlist_id VARCHAR(11), genre TEXT)",
            "CREATE TABLE IF NOT EXISTS downloaded_videos (youtube_video_id VARCHAR(11), playlist_id VARCHAR(11), failed BOOLEAN)",
            "CREATE UNIQUE INDEX IF NOT EXISTS playlists_playlists_id_index ON playlists (playlist_id)",
            "CREATE UNIQUE INDEX IF NOT EXISTS downloaded_videos_video_id ON downloaded_videos (youtube_video_id)"
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

    pub fn get_downloaded_videos_from_playlist(
        &self,
        playlist_id: String,
    ) -> Result<Vec<String>, String> {
        //create query
        let query = "SELECT * FROM downloaded_videos WHERE playlist_id like ?1";

        //list of youtube video ids
        let mut youtube_video_ids: Vec<String> = Vec::new();

        //prepare statment
        let mut statement = match self.connection.prepare(query) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not create prepared statement in get downloaded videos : {}: {}",
                    query, e
                ));
            }
        };

        //execute query, map resulting rows
        let videos = match statement.query_map(params![playlist_id], |row| {
            let row_str: String = row.get(0)?;
            Ok(row_str)
        }) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("Could not execute prepared statement and collect row information in get downloaded videos: {}: {}", query, e));
            }
        };

        for video_result in videos {
            let video = match video_result {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!("Error fetching a row for prepared statement {} in get downloaded videos: {}", query, e));
                }
            };

            youtube_video_ids.push(video);
        }

        return Ok(youtube_video_ids);
    }

    /// Put downloaded video information into database
    ///   If already exists, will silently ignore
    pub fn put_downloaded_video(
        &self,
        playlist_id: String,
        video_id: String,
        failed: bool,
    ) -> Result<(), String> {
        //create query
        let query = "INSERT INTO downloaded_videos VALUES (?1, ?2, ?3) ON CONFLICT(?1) DO NOTHING";

        // execute statement
        let statement_result = self
            .connection
            .execute(&query, params![video_id, playlist_id, failed]);

        //execute query, parse result
        match statement_result {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "Could not execute put downloaded videos query: {}: {}",
                    query, e
                ));
            }
        }

        return Ok(());
    }

    /// Put playlist information into database
    ///   If already exists, will silently ignore
    pub fn put_playlist(&self, playlist_id: String, genre: String) -> Result<(), String> {
        //create query
        let query = "INSERT OR REPLACE INTO playlists(playlist_id, genre) VALUES (?1, ?2)";

        //generate prepared statment
        let _ = match self.connection.execute(&query, params![playlist_id, genre]) {
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
    pub fn delete_playlist(&self, playlist_id: String) -> Result<(), String> {
        //delete all downloads videos from the downloaded videos table with the playlist id
        let query = "DELETE FROM downloaded_videos WHERE playlist_id = ?1";

        // execute statement
        let _ = match self.connection.execute(&query, params![playlist_id]) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not execute delete downloaded videos of delete playlist query: {}: {}",
                    query, e
                ));
            }
        };

        //delete the playlist from the playlists database
        let query = "DELETE FROM playlists WHERE playlist_id = ?1";

        // execute statement
        let _ = match self.connection.execute(&query, params![playlist_id]) {
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
    pub fn get_all_playlists(&self) -> Result<Vec<(String, String)>, String> {
        //create query
        let query = "SELECT * FROM playlists";

        //list of playlist ids
        let mut playlists: Vec<(String, String)> = Vec::new();

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
            let playlist_id: String = row.get(0)?;
            let genre: String = row.get(1)?;

            Ok((playlist_id, genre))
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
                    return Err(format!("Error fetching a row for prepared statement {} in get downloaded videos: {}", query, e));
                }
            };

            playlists.push(playlist);
        }

        return Ok(playlists);
    }
}
