use sqlite;

pub struct DatabaseConnection {
    connection_obj: sqlite::Connection
}

fn init_database(relative_database_path: &str) -> Result<DatabaseConnection, String>{
    //initialize the connection
    let connection = sqlite::open("../data/sqlite").unwrap();

    //create / re-establish presence of necessary tables 
    let create_table_queries = [
        "CREATE TABLE IF NOT EXISTS playlists (playlist_id TEXT, genre TEXT)",
        "CREATE TABLE IF NOT EXISTS downloaded_songs (youtube_video_id TEXT, failed BOOLEAN)"
    ];

    //for each create table query
    for create_table_query in create_table_queries.iter() {
        //execute the query
        let mut statement = match connection.prepare(create_table_query) {
            Ok(some) => some,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        let statement_result = statement.next();

        match statement_result {
            Ok(_) => None,
            Err(e) => {
                return format!("Could not execute create table statement: {}", create_table_query);
            }
        }
    }

    return Ok(DatabaseConnection {
        connection_obj: connection
    });
 

}

impl DatabaseConnection {
    fn get_downloaded_videos(playlist_id: String) -> Result<Vec<String>, String> {
        todo!();
    }

    fn put_downloaded_video(playlist_id: String, video_id: String, failed: bool) -> Result<_, String> {
        todo!();
    }

    fn put_playlist(playlist_id: String, genre: String) -> Result<_, String> {
        todo!();
    }

    fn get_playlists() -> Result<Vec<String>, String> {
        todo!();
    }
}