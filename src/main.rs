use std::path::PathBuf;
use rustube::{self, tokio::stream};
use crate::audio_extractor::{EmptyAudioExtractor, InitializedAudioExtractor, FinishedAudioExtractor};
use crate::title_extractor::{EmptyTitleExtractor, InitializedTitleExtractor, FinishedTitleExtractor};

pub mod audio_extractor;
pub mod title_extractor;

//TODO create directory if deleted
fn main() {
    /* 
    let url: &str= "https://www.youtube.com/watch?v=y-bt-KUb0Nc&list=PL5MlDErkUccBvnU74GlkuNmCifkfLf1o8";
    let path_to_video = rustube::blocking::download_worst_quality(url).unwrap();
    let str_path_to_video = path_to_video.as_os_str().to_str().unwrap();
    */
    let audio_extractor: InitializedAudioExtractor = EmptyAudioExtractor::init("y-bt-KUb0Nc");
    let audio_extractor: FinishedAudioExtractor = audio_extractor.download().unwrap();

    let title_extractor: InitializedTitleExtractor = EmptyTitleExtractor::init(audio_extractor.title().clone());
    let title_extractor: FinishedTitleExtractor = title_extractor.extract_from_title().unwrap();

    println!("the total song title is {}", audio_extractor.title().clone());

    println!("song is at {} with title {}, name {}, and artist {} by video author {}", audio_extractor.write_path().as_os_str().to_str().unwrap(), audio_extractor.title(), title_extractor.name(), title_extractor.artist(), audio_extractor.author()); 
    /*
    let url: &str = "https://www.youtube.com/watch?v=y-bt-KUb0Nc&list=PL5MlDErkUccBvnU74GlkuNmCifkfLf1o8";

    /*
    let id = rustube::Id::from_raw(url).unwrap();

    {    
        for strm in rustube::blocking::Video::from_id(id).unwrap().streams().iter() {
            println!("{}", strm.mime);
        }
    }*/

    //get current working direcotry
    let current_working_directory: PathBuf = std::env::current_dir().unwrap();

    let relative_audio_directory: &str= "/data/audio";

    //append relative path to current working directory
    let audio_directory: PathBuf = current_working_directory.join("data").join("audio");

    let id = rustube::Id::from_raw(url).unwrap();

    let path_to_video = rustube::blocking::Video::from_id(id.into_owned()).unwrap()
        .streams()
        .iter()
        .filter(|stream| stream.includes_audio_track && !stream.includes_video_track)
        .find(|stream| stream.mime.to_string() == "audio/mp4")
        .unwrap()
        .blocking_download()
        .unwrap();

    let str_path_to_video = path_to_video.as_os_str().to_str().unwrap();
    */

    //get video information

    //get audio from video

    //println!("{str_path_to_video}");
}

/*anytime after 2pm november 17th or anytime that weekend works for me. look foward to hearing back from you soon. */