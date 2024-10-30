use crate::audio_extractor::{
    EmptyAudioExtractor, FinishedAudioExtractor, InitializedAudioExtractor,
};
use crate::audio_tag_appender::{
    EmptyAudioTagAppender, FinalizedAudioTagAppender, InitializedAudioTagAppender,
};
use crate::title_extractor::{
    EmptyTitleExtractor, FinishedTitleExtractor, InitializedTitleExtractor,
};
use database::{Database, UninitializedDatabase};
use lazy_static::lazy_static;
use rustube::{self, tokio::stream};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::current;
use youtube_playlist_extractor::get_playlist_videos;

pub mod audio_extractor;
pub mod audio_tag_appender;
pub mod command_line_extractor;
pub mod database;
pub mod process;
pub mod settings_parser;
pub mod title_extractor;
pub mod youtube_playlist_extractor;

use audiotags::{MimeType, Tag};

//TODO create directory if deleted
fn main() {
    // create contexts
    //let mut database_context = Database::default();

    //parse command line arguments
    //let try_failed = command_line_extractor::parse_args(&mut database_context).unwrap();

    let playlist_videos = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_playlist_videos(
            "PL5MlDErkUccBvnU74GlkuNmCifkfLf1o8".to_owned(),
        ))
        .unwrap();

    for playlist_video in playlist_videos {
        println!("{}", playlist_video.title);
    }

    /*let audio_extractor: InitializedAudioExtractor = EmptyAudioExtractor::init("y-bt-KUb0Nc");
    let audio_extractor: FinishedAudioExtractor = audio_extractor.download().unwrap();

    let title_extractor: InitializedTitleExtractor = EmptyTitleExtractor::init(audio_extractor.title().clone());
    let title_extractor: FinishedTitleExtractor = title_extractor.extract_from_title().unwrap();

    println!("the total song title is {}", audio_extractor.title().clone());

    println!("song is at {} with title {}, name {}, and artist {} by video author {}", audio_extractor.write_path().as_os_str().to_str().unwrap(), audio_extractor.title(), title_extractor.name(), title_extractor.artist(), audio_extractor.author());

    let tag_appender: InitializedAudioTagAppender = EmptyAudioTagAppender::init(&audio_extractor);
    let tag_appender: FinalizedAudioTagAppender = tag_appender.append_metadata().unwrap();


    //parse playlist file

    //add and remove playlists as found in file

    //for each playlist
        //download each new song

    //update database*/
}

/*anytime after 2pm november 17th or anytime that weekend works for me. look foward to hearing back from you soon. */
