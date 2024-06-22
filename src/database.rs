use sqlite::{self, State};

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
    pub fn implement(&mut self) -> Result<(), String> {
        match self.state {
            DatabaseState::UninitializedDatabase(_) => {
                self.state = DatabaseState::InitializedDatabase(self.state)?
            }
            DatabaseState::InitializedDatabase(_) => Ok(())
        }
    }
}

impl InitializedDatabase {
    /// Create an InitializedDatabase from a UnintializedDatabase
    pub fn new(_: UninitializedDatabase) -> Result<InitializedDatabase, String> {
        //initialize the connection
        let connection = sqlite::open("../data/sqlite").unwrap();

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