use getset::Getters;
use rusty_ytdl::{FFmpegArgs, Video, VideoOptions, VideoQuality, VideoSearchOptions};
use std::{io::Write, path::PathBuf};
use uuid::Uuid;

const DATA_DIRECTORY_NAME: &str = "data";
const TEMP_AUDIO_DIRECTORY_NAME: &str = "audio";

pub struct EmptyAudioExtractor;

pub struct InitializedAudioExtractor {
    id: String,
}

#[derive(Getters)]
pub struct FinishedAudioExtractor {
    #[getset(get = "pub")]
    write_path: PathBuf,

    #[getset(get = "pub")]
    title: String,

    #[getset(get = "pub")]
    author: String,
}

/// Contains the file with the metadata set after title extractor
pub struct AudioExtractorPlayableMedia {
    _write_path: PathBuf,
}

impl EmptyAudioExtractor {
    pub fn init(id: &str) -> InitializedAudioExtractor {
        let updated_self: InitializedAudioExtractor =
            InitializedAudioExtractor { id: id.to_string() };

        return updated_self;
    }
}

//TODO add audio extension type to finished audio extactor, maybe Minetype enum
impl InitializedAudioExtractor {
    /*pub async fn download(&self) -> Result<FinishedAudioExtractor, String> {
        // set directory that the file will be written to
        let current_working_directory: PathBuf = match std::env::current_dir() {
            Ok(val) => val,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let path = current_working_directory
            .join(DATA_DIRECTORY_NAME)
            .join(TEMP_AUDIO_DIRECTORY_NAME);

        //create directories along 'path' if they already do not exist
        match std::fs::create_dir_all(&path) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Failed to create directories: {:?}", e));
            }
        }

        // get rusttube id object from raw video id
        let id = match rustube::Id::from_str(&self.id) {
            Ok(some) => some,
            Err(e) => {
                return Err(format!("id {} is not valid: {}", self.id, e));
            }
        };

        //get video object
        let video_obj = match rustube::Video::from_id(id.into_owned()).await {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "could not create video object from id for reason {e}"
                ));
            }
        };

        //get desired audio stream
        let audio_stream_list: Vec<&rustube::Stream> = video_obj
            .streams()
            .iter()
            .filter(|stream| stream.includes_audio_track && !stream.includes_video_track)
            .collect();

        //set a temporary file name with hopes of no collision, if this is parallelized
        let mut file_name = "temp_".to_string();
        file_name = file_name + &Uuid::new_v4().to_string();

        //attempt to find the mp4 audio stream
        let mut audio_stream = audio_stream_list
            .clone()
            .into_iter()
            .find(|stream| stream.mime.to_string() == "audio/mp4");
        {
            let mut file_extension: &str = ".m4a";

            //if desired audio stream is not found, try other acceptable ones
            if audio_stream == None {
                audio_stream = audio_stream_list
                    .clone()
                    .into_iter()
                    .find(|stream| stream.mime.to_string() == "audio/mp3");

                file_extension = ".mp4";
            }

            if audio_stream == None {
                audio_stream = audio_stream_list
                    .clone()
                    .into_iter()
                    .find(|stream| stream.mime.to_string() == "audio/wav");

                file_extension = ".wav";
            }
            //future supported formats could be mp3 or wave, but have to adapt audio tag appender

            if audio_stream == None {
                return Err("Could not find valid audio format".to_string());
            }

            //appemd file extensions since we now know the audio format
            file_name.push_str(file_extension);
        }

        //append file name to path for full write path
        let path = path.join(file_name);

        //attempt to download video
        let path_to_video = match audio_stream {
            Some(stream) => {
                //audio stream found, download audio
                match stream.download_to(path.clone()).await {
                    Ok(_) => path,
                    Err(e) => {
                        //TODO remove unwrap here
                        return Err(format!(
                            "Could not download video to {}: {}",
                            path.as_os_str().to_str().unwrap_or_default(),
                            e
                        ));
                    }
                }
            }
            None => {
                //could not find wav audio stream for video, skipping video, marking as failed
                return Err("Could not find valid audio stream".to_string());
            }
        };

        //get video information
        let video_details: &rustube::VideoDetails = video_obj
            .video_info()
            .player_response
            .video_details
            .as_ref();

        //create copies of needed data
        let video_title = video_details.title.clone();
        let video_author = video_details.author.clone();

        // construct finished audio object
        let finished_audio_obj: FinishedAudioExtractor = FinishedAudioExtractor {
            write_path: path_to_video.clone(),
            title: video_title,
            author: video_author,
        };

        return Ok(finished_audio_obj);
    }
    */

    pub async fn download(&self) -> Result<FinishedAudioExtractor, String> {
        // set directory that the file will be written to
        let current_working_directory: PathBuf = match std::env::current_dir() {
            Ok(val) => val,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let path = current_working_directory
            .join(DATA_DIRECTORY_NAME)
            .join(TEMP_AUDIO_DIRECTORY_NAME)
            .join("temp.mp3");

        //create directories along 'path' if they already do not exist
        match std::fs::create_dir_all(&path.parent().unwrap().to_path_buf()) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Failed to create directories: {:?}", e));
            }
        }

        let video_options = VideoOptions {
            quality: VideoQuality::HighestAudio,
            filter: VideoSearchOptions::Audio,
            ..Default::default()
        };

        let video = Video::new_with_options(self.id.to_owned(), video_options).unwrap();

        /*let stream = video
        .stream_with_ffmpeg(Some(FFmpegArgs {
            format: Some("mp3".to_string()),
            audio_filter: None,
            video_filter: None,
        }))
        .await
        .unwrap();*/

        let stream = video.stream().await.unwrap();

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();

        while let Some(chunk) = stream.chunk().await.unwrap() {
            file.write_all(&chunk).unwrap();
        }

        let finished_audio_extractor = FinishedAudioExtractor {
            write_path: path,
            title: "".to_string(),
            author: "".to_string(),
        };

        return Ok(finished_audio_extractor);
    }
}

// https://github.com/Mithronn/rusty_ytdl/tree/main
// https://crates.io/crates/A2VConverter
