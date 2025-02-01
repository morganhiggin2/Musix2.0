use database::Database;
use environment_extractor::get_environment_variables;
use music_sources::{soundcloud_service::SoundcloudMusicService, MusicSource};

pub mod audio_tag_appender;
pub mod command_line_runtime;
pub mod database;
pub mod environment_extractor;
pub mod music_sources;
pub mod post_processor;
pub mod s3_service;
pub mod settings_parser;
pub mod title_extractor;
pub mod url_enforcer;
pub mod yt_dlp_caller;

fn main() {
    // Get environment variables
    let environment_variables = get_environment_variables().unwrap();

    // create contexts
    let mut database_context = Database::default();

    //parse command line arguments and execute them
    command_line_runtime::parse_args(&mut database_context, &environment_variables).unwrap();

    //println!("{}", downloaded_song.title);
}

// TODO have the database file live on s3 for maintainability, as the docker image won't have to reset
//TODO implement funcationality to create the database file in s3 if it does not already exist yet
// - however, this exposes the risk of now it can create an object in s3, have to be specific about which object it can create
// - already have to have the bucket exist
// TODO migrate to anyhow
// TODO unit tests for pasing using actual sample data from requests
