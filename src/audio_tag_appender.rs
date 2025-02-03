use audiotags::Tag;
use std::fs::OpenOptions;

use crate::music_sources;

pub fn append_metadata(song_information: &music_sources::DownloadedSong) -> Result<(), String> {
    let mut audio_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(&song_information.file_location)
    {
        Ok(audio_file) => audio_file,
        Err(e) => {
            return Err(format!("Could not get file: {}", e));
        }
    };

    //read metatdata from existing file
    let mut current_tags = match Tag::default().read_from_path(&song_information.file_location) {
        Ok(val) => val,
        Err(e) => {
            return Err(format!("Could not get tags from audio file: {}", e));
        }
    };

    //set appropriate tags
    current_tags.set_title(&song_information.title);
    current_tags.set_artist(&song_information.artist);
    current_tags.set_genre(&song_information.genre);

    match current_tags.write_to(&mut audio_file) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Cannot write to audio file: {}", e));
        }
    };

    return Ok(());
}
