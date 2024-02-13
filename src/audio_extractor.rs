use std::{path::PathBuf, sync::Arc};
use rustube::{VideoDescrambler, VideoFetcher, VideoInfo, Video, video_info::player_response::video_details};
use uuid::Uuid;
use getset::Getters;

const DATA_DIRECTORY_NAME: &str = "data";
const TEMP_AUDIO_DIRECTORY_NAME: &str = "audio";

pub struct EmptyAudioExtractor;

pub struct InitializedAudioExtractor
{
    id: String
}

#[derive(Getters)]
pub struct FinishedAudioExtractor
{
    #[getset(get = "pub")]
    write_path: PathBuf,

    #[getset(get = "pub")]
    title: String,

    #[getset(get = "pub")]
    author: String
}

/// Contains the file with the metadata set after title extractor
pub struct AudioExtractorPlayableMedia {
    write_path: PathBuf
}

impl EmptyAudioExtractor {
    pub fn init(id: &str) -> InitializedAudioExtractor {
        let updated_self : InitializedAudioExtractor = InitializedAudioExtractor {
            id: id.to_string()
        };

        return updated_self
    }
}

impl InitializedAudioExtractor {
    pub fn download(&self) -> Result<FinishedAudioExtractor, String> {
        // set path

        //TODO get rid of unwrap
        let current_working_directory: PathBuf = std::env::current_dir().unwrap();
        let path = current_working_directory.join(DATA_DIRECTORY_NAME).join(TEMP_AUDIO_DIRECTORY_NAME);

        //TODO append file directory
        //TODO append tempoary file name with random uuid added, then move to final destination with final name and metadata

        // get rusttube id object from raw id
        let id = match rustube::Id::from_str(&self.id) {
            Ok(some) => {
                some 
            }
            Err(e) => {
                return Err(format!("id {} is not valid: {}", self.id, e));
            }
        };
         
        //get video object 
        let video_obj = match rustube::blocking::Video::from_id(id.into_owned()) {
            Ok(some) => {
                some
            }
            Err(e) => {
                return Err(format!("could not create video object from id for reason {e}"));
            }
        };
        //get desired audio stream
        let audio_stream_list : Vec<&rustube::Stream> = video_obj
        .streams()
        .iter()
        .filter(|stream| stream.includes_audio_track && !stream.includes_video_track)
        .collect();

        /*let audio_stream_position = audio_stream_list.iter().position(|stream| stream.mime.to_string() == "audio/mp4").unwrap_or(0);
        audio_stream = audio_stream_list.get(audio_stream_position).copied();*/
        let mut audio_stream = audio_stream_list.clone().into_iter().find(|stream| stream.mime.to_string() == "audio/mp4");

        let mut file_name = "temp_".to_string();
        file_name = file_name + &Uuid::new_v4().to_string();

        //TODO clean, look sloppy
        //if desired audio stream is not found, try other acceptable ones
        if audio_stream == None {
            audio_stream = audio_stream_list.clone().into_iter().find(|stream| stream.mime.to_string() == "audio/mp3");
        }
        else {
            //append extension
            file_name.push_str(".mp4");
        }

        if audio_stream == None {
            audio_stream = audio_stream_list.clone().into_iter().find(|stream| stream.mime.to_string() == "audio/wav");
        }
        else {
            file_name.push_str(".mp3");
        }

        if audio_stream == None {
            return Err("Could not find valid audio format".to_string());
        }
        else {
            file_name.push_str(".wav");
        }

        //TODO change
        let file_name = "file_1.mp4";
        //append file name to path
        let path = path.join(file_name);

        //attempt to download video
        let path_to_video = match audio_stream {
            Some(stream) => {
                //audio stream found, download audio
                match stream.blocking_download_to(path.clone()) {
                    Ok(_) => {
                        path
                    }
                    Err(e) => {
                        return Err(format!("Could not download video to {}: {}", path.as_os_str().to_str().unwrap(), e));
                    }
                }
            }
            None => {
                //could not find wav audio stream for video, skipping video, marking as failed 
                return Err("Could not find valid audio stream".to_string())
            }
        };

        // get os path to video
        //let str_path_to_video = path_to_video.as_os_str().to_str().unwrap();

        //get video information
        let video_info: &VideoInfo = video_obj.video_info();

        let video_details: &rustube::VideoDetails = video_info.player_response.video_details.as_ref();
    
        //create copies of needed data
        let video_title = video_details.title.clone();
        let video_author = video_details.author.clone();

        // construct finished audio object
        let finished_audio_obj: FinishedAudioExtractor = FinishedAudioExtractor { write_path: path_to_video.clone(), title: video_title, author: video_author };

        return Ok(finished_audio_obj);
    }
}

//factory that has
//url()
//download()

//returning title and other data object with it