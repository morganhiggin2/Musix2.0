use rusqlite::{self, params, Row};

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
    fn initialize_if_required(&mut self) -> Result<(), String> {
        match &mut self.state {
            DatabaseState::UninitializedDatabase(state) => {
                let new_state = match InitializedDatabase::new(state) {
                    Ok(some) => DatabaseState::InitializedDatabase(some),
                    Err(e) => return Err(e),
                };

                self.state = new_state;

                return Ok(());
            }
            DatabaseState::InitializedDatabase(_) => return Ok(()),
        };
    }

    fn get_initialized_state_always(&mut self) -> Result<&mut InitializedDatabase, String> {
        //TODO can we do this inside the match to valid the panic call? using ref maybe to inspect and condition
        
        self.initialize_if_required()?;

        // We know for a fact that by this point the state will be an initiailized database
        match &mut self.state {
            DatabaseState::UninitializedDatabase(state) => {
                // And thus this path should never be traversed
                panic!();
            }
            DatabaseState::InitializedDatabase(state) => {
                return Ok(state);
            }
        };
    }

    // Wrapper for initiazlied database calls
    pub fn get_downloaded_videos(&mut self, playlist_id: String) -> Result<Vec<String>, String> {
        let initialzied_database = self.get_initialized_state_always()?;

        return initialzied_database.get_downloaded_videos(playlist_id);
    }

    pub fn put_downloaded_video(
        &mut self,
        playlist_id: String,
        video_id: String,
        failed: bool,
    ) -> Result<(), String> {
        let initialzied_database = self.get_initialized_state_always()?;

        return initialzied_database.put_downloaded_video(playlist_id, video_id, failed);
    }

    pub fn put_playlist(&mut self, playlist_id: String, genre: String) -> Result<(), String> {
        let initialzied_database = self.get_initialized_state_always()?;

        return initialzied_database.put_playlist(playlist_id, genre);
    }

    pub fn delete_playlist(&mut self, playlist_id: String) -> Result<(), String> {
        let initialzied_database = self.get_initialized_state_always()?;

        return initialzied_database.delete_playlist(playlist_id);
    }

    pub fn get_all_playlists(&mut self) -> Result<Vec<String>, String> {
        let initialzied_database = self.get_initialized_state_always()?;

        return initialzied_database.get_all_playlists();
    }
}

impl InitializedDatabase {
    /// Create an InitializedDatabase from a UnintializedDatabase
    pub fn new(_: &mut UninitializedDatabase) -> Result<InitializedDatabase, String> {
        match std::fs::create_dir_all("data/database") {
            Ok(()) => (),
            Err(e) => {
                return Err(format!(
                    "Could not create files for database 'data/database': {}",
                    e
                ));
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

        return Ok(InitializedDatabase {
            connection: connection,
        });
    }

    pub fn get_downloaded_videos(&self, playlist_id: String) -> Result<Vec<String>, String> {
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
        let videos = match statement.query_map([], |row| {
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
        // TODO error not deduplicating based on playlist id

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

    /// Delete playlist from database
    ///     If the playlist does not exist, will do nothing
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
            let row_str: String = row.get(0)?;
            Ok(row_str)
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
