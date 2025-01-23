use audiotags::Tag;
use std::fs::OpenOptions;

pub struct EmptyAudioTagAppender;

pub struct InitializedAudioTagAppender<'a> {
    audio_extractor: &'a FinishedAudioExtractor,
}

pub struct FinalizedAudioTagAppender {}

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
    ) -> Result<FinalizedAudioTagAppender, String> {
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

        return Ok(FinalizedAudioTagAppender {});
    }
}
