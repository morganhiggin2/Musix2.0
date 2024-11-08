use database::Database;

pub mod audio_extractor;
pub mod audio_tag_appender;
pub mod command_line_extractor;
pub mod database;
pub mod process;
pub mod settings_parser;
pub mod title_extractor;
pub mod youtube_playlist_extractor;

//TODO create directory if deleted
#[tokio::main]
async fn main() {
    // create contexts
    let mut database_context = Database::default();

    //parse command line arguments
    command_line_extractor::parse_args(&mut database_context)
        .await
        .unwrap();
}
