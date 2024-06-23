use sqlite::{self, Connection, OpenFlags, State};

pub struct Database {
    state: DatabaseState 
}

pub enum DatabaseState {
    UninitializedDatabase(UninitializedDatabase),
    InitializedDatabase(InitializedDatabase)
}

#[derive(Default)]
pub struct UninitializedDatabase {

}

pub struct InitializedDatabase {
    connection: sqlite::Connection
}

impl Default for Database {
    fn default() -> Self {
        return Database {
            state: DatabaseState::UninitializedDatabase(UninitializedDatabase::default())
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
                    Err(e) => {
                        return Err(e)
                    }
                };

                self.state = new_state;

                return Ok(());
            }
            DatabaseState::InitializedDatabase(_) => {
                return Ok(())
            } 
        };
    }

    fn get_initialized_state_always(&mut self) -> Result<&mut InitializedDatabase, String> {
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

    pub fn put_downloaded_video(&mut self, playlist_id: String, video_id: String, failed: bool) -> Result<(), String> {
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
        //initialize the connection
        let open_connection_flags = OpenFlags::new().with_create().with_read_write();
        let connection = Connection::open_with_flags("data/database/sqlite.db", open_connection_flags).unwrap();

        //create / re-establish presence of necessary tables 
        let create_table_queries = [
            "CREATE TABLE IF NOT EXISTS playlists (playlist_id VARCHAR(11), genre TEXT)",
            "CREATE TABLE IF NOT EXISTS downloaded_videos (youtube_video_id VARCHAR(11), playlist_id VARCHAR(11), failed BOOLEAN)"
        ];

        //for each create table query
        for create_table_query in create_table_queries.iter() {
            // statement
            let statement_result = connection.execute(create_table_query); 

            match statement_result {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!("Could not  create table statement: {}: {}", create_table_query, e));
                }
            }
        }

        return Ok(InitializedDatabase {
            connection: connection
        });
    }

    pub fn get_downloaded_videos(&self, playlist_id: String) -> Result<Vec<String>, String> {
        //create query
        let query = format!("SELECT * FROM downloaded_videos WHERE playlist_id like {}", playlist_id);

        //generate prepared statment
        let mut statement = match self.connection.prepare(query.to_owned()) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("Error creating perpared statement {}: {}", query, e));
            }
        };

        //list of youtube video ids
        let mut youtube_video_ids: Vec<String> = Vec::new();

        // prepared statement, process rows
        while let Ok(State::Row) = statement.next(){
            //get youtube video id column value from row
            let youtube_video_id = match statement.read::<String, _>("youtube_video_id") {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!("Error getting column \'youtube_video_id\' from result set of query {}: {}", query, e));
                }
            };

            //add video id to list
            youtube_video_ids.push(youtube_video_id);
        }

        return Ok(youtube_video_ids);
    }

    /// Put downloaded video information into database
    ///   If already exists, will silently ignore
    pub fn put_downloaded_video(&self, playlist_id: String, video_id: String, failed: bool) -> Result<(), String> {
        //create query
        let query = format!("INSERT INTO downloaded_videos VALUES ({}, {}, {}) ON CONFLICT({video_id}) DO NOTHING", video_id, playlist_id, failed);

        // execute statement
        let statement_result = self.connection.execute(&query); 

        //execute query, parse result
        match statement_result {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Could not execute put downloaded videos query: {}: {}", query, e));
            }
        }

        return Ok(());
    }

    /// Put playlist information into database
    ///   If already exists, will silently ignore
    pub fn put_playlist(&self, playlist_id: String, genre: String) -> Result<(), String> {
        //create query
        let query = format!("INSERT INTO playlists VALUES ({}, {}) ON CONFLICT({playlist_id}) DO NOTHING", playlist_id, genre);

        //generate statement
        let statement_result = self.connection.execute(&query); 

        //execute query, parse result
        match statement_result {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Could not execute put downloaded videos query: {}: {}", query, e));
            }
        }

        return Ok(());
    }

    /// Delete playlist from database
    ///     If the playlist does not exist, will do nothing 
    pub fn delete_playlist(&self, playlist_id: String) -> Result<(), String> {
        //delete all downloads videos from the downloaded videos table with the playlist id
        let query = format!("DELETE FROM downloaded_videos WHERE playlist_id = {}", playlist_id);

        // execute statement
        let statement_result = self.connection.execute(&query); 

        //execute query, parse result
        match statement_result {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Could not execute delete downloaded videos of delete playlist query: {}: {}", query, e));
            }
        }

        //delete the playlist from the playlists database
        let query = format!("DELETE FROM playlists WHERE playlist_id = {}", playlist_id);
        
        // execute statement
        let statement_result = self.connection.execute(&query); 

        //execute query, parse result
        match statement_result {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Could not execute delete playst of delete playlist query: {}: {}", query, e));
            }
        }

        return Ok(());
    }

    pub fn get_all_playlists(&self) -> Result<Vec<String>, String> {
        //create query
        let query = "SELECT * FROM playlists";
        //generate prepared statment
        let mut statement = match self.connection.prepare(query) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("Error creating perpared statement {}: {}", query, e));
            }
        };

        //list of playlist ids
        let mut playlist_ids: Vec<String> = Vec::new();

        // prepared statement, process rows
        while let Ok(State::Row) = statement.next(){
            //get youtube video id column value from row
            let playlist_id = match statement.read::<String, _>("playlist_id") {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!("Error getting column \'playlist_id\' from result set of query {}: {}", query, e));
                }
            };

            //add video id to list
            playlist_ids.push(playlist_id);
        }

        return Ok(playlist_ids);
    }
}