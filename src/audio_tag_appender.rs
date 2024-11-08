use audiotags::Tag;
use std::fs::OpenOptions;

use crate::audio_extractor::FinishedAudioExtractor;

pub struct EmptyAudioTagAppender;

pub struct InitializedAudioTagAppender<'a> {
    audio_extractor: &'a FinishedAudioExtractor,
}

pub struct FinalizedAudioTagAppender<'a> {
    audio_extractor: &'a FinishedAudioExtractor,
}

impl EmptyAudioTagAppender {
    pub fn init(finished_audio_extractor: &FinishedAudioExtractor) -> InitializedAudioTagAppender {
        return InitializedAudioTagAppender {
            audio_extractor: finished_audio_extractor,
        };
    }
}

impl<'a> InitializedAudioTagAppender<'a> {
    pub fn append_metadata(
        self: InitializedAudioTagAppender<'a>,
        genre: &String,
    ) -> Result<FinalizedAudioTagAppender<'a>, String> {
        let mut audio_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.audio_extractor.write_path())
        {
            Ok(audio_file) => audio_file,
            Err(e) => {
                return Err(format!("Could not get file: {}", e));
            }
        };

        //TODO delete
        /*let write_path_str = match self.audio_extractor.write_path().to_str() {
            Some(str_path) => str_path,
            None => {
                return Err("Cannot stringify write path for audio file".to_string());
            }
        };*/

        //read metatdata from existing file
        let mut current_tags =
            match Tag::default().read_from_path(self.audio_extractor.write_path()) {
                Ok(val) => val,
                Err(e) => {
                    return Err(format!("Could not get tags from audio file: {}", e));
                }
            };

        //set appropriate tags
        current_tags.set_title(&self.audio_extractor.title());
        current_tags.set_artist(&self.audio_extractor.author());
        current_tags.set_genre(genre);

        match current_tags.write_to(&mut audio_file) {
            Ok(_) => {
                let title = current_tags.title().unwrap();
                println!("wrote to path with {title}");
            }
            Err(e) => {
                return Err(format!("Cannot write to audio file: {}", e));
            }
        }

        return Ok(FinalizedAudioTagAppender {
            audio_extractor: self.audio_extractor,
        });
    }
}
