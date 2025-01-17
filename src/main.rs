use audio_extractor::EmptyAudioExtractor;
use database::Database;
use environment_extractor::get_environment_variables;

pub mod audio_extractor;
pub mod audio_tag_appender;
pub mod command_line_runtime;
pub mod database;
pub mod environment_extractor;
pub mod s3_service;
pub mod settings_parser;
pub mod title_extractor;
pub mod youtube_playlist_extractor;

//TODO create directory if deleted
// TODO get rid of tokio if possible
#[tokio::main]
async fn main() {
    // Get environment variables
    let environment_variables = get_environment_variables().unwrap();

    // create contexts
    let mut database_context = Database::default();

    //parse command line arguments and execute them
    command_line_runtime::parse_args(&mut database_context, &environment_variables)
        .await
        .unwrap();

    let audio_extractor = EmptyAudioExtractor::init("FZ8BxMU3BYc");
    audio_extractor.download().await.unwrap();
}

// TODO have the database file live on s3 for maintainability, as the docker image won't have to reset
