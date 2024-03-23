use sqlite;

pub struct DatabaseConnection {
    connection: sqlite::Connection
}



impl DatabaseConnection {
    fn new(relative_database_path: &str) -> Result<DatabaseConnection, String>{
        //initialize the connection
        let connection = sqlite::open("../data/sqlite").unwrap();

        //create / re-establish presence of necessary tables 
        let create_table_queries = [
            "CREATE TABLE IF NOT EXISTS playlists (playlist_id VARCHAR(11), genre TEXT)",
            "CREATE TABLE IF NOT EXISTS downloaded_videos (youtube_video_id VARCHAR(11), playlist_id VARCHAR(11), failed BOOLEAN)"
        ];

        //for each create table query
        for create_table_query in create_table_queries.iter() {
            //execute the query
            let mut statement = match connection.prepare(create_table_query) {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!("Error in creating prepared statement {}: {}", create_table_query, e.to_string()));
                }
            };

            //execute statement
            let statement_result = connection.execute(statement); 

            match statement_result {
                Ok(_) => None,
                Err(e) => {
                    return format!("Could not execute create table statement: {}: {}", create_table_query, e);
                }
            }
        }

        return Ok(DatabaseConnection {
            connection: connection
        });
    }

    fn get_downloaded_videos(self, playlist_id: String) -> Result<Vec<String>, String> {
        //create query
        let query = "SELECT * FROM downloaded_videos WHERE playlist_id like ?";
        //generate prepared statment
        let mut statement = match self.connection.prepare(query) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("Error creating perpared statement {}: {}", query, e));
            }
        };

        //fill in placeholders in prepared statement
        match statement.bind((1, playlist_id)) {
            Ok(_) => _,
            Err(e) => {
                return Err(format!("Error binding value {} to prepared statement {}: {}", playlist_id, query, e));
            }
        };

        //list of youtube video ids
        let mut youtube_video_ids: Vec<String> = Vec::new();

        //execute prepared statement, process rows
        while let Ok(row) = statement.next(){
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

    fn put_downloaded_video(self, playlist_id: String, video_id: String, failed: bool) -> Result<_, String> {
        todo!();
    }

    fn put_playlist(self, playlist_id: String, genre: String) -> Result<_, String> {
        todo!();
    }

    fn get_playlists(self) -> Result<Vec<String>, String> {
        todo!();
    }
}