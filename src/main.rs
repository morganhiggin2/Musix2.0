use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::current;
use database::{Database, UninitializedDatabase};
use rustube::{self, tokio::stream};
use crate::audio_extractor::{EmptyAudioExtractor, InitializedAudioExtractor, FinishedAudioExtractor};
use crate::title_extractor::{EmptyTitleExtractor, InitializedTitleExtractor, FinishedTitleExtractor};
use crate::audio_tag_appender::{EmptyAudioTagAppender, InitializedAudioTagAppender, FinalizedAudioTagAppender};
use lazy_static::lazy_static;

pub mod audio_extractor;
pub mod title_extractor;
pub mod audio_tag_appender;
pub mod youtube_playlist_extractor;
pub mod database;
pub mod command_line_extractor;
pub mod settings_parser;
pub mod process;

use audiotags::{Tag, MimeType};

//TODO create directory if deleted
fn main() {
    // create contexts
    let mut database_context = Database::default();

    //parse command line arguments
    let try_failed = command_line_extractor::parse_args(&mut database_context).unwrap();
    
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