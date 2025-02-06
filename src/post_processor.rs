use std::{path::PathBuf, str::FromStr};

use crate::{audio_tag_appender, music_sources::DownloadedSong};

// manage downloaded song, return new location
// returns the path to the new song
pub fn post_process_downloaded_song(downloaded_song: DownloadedSong) -> Result<PathBuf, String> {
    // add metadata to song file
    audio_tag_appender::append_metadata(&downloaded_song)?;

    // rename file to include the artist and name of the song
    let renamed_file_path = match downloaded_song.file_location.parent() {
        Some(path_buf) => PathBuf::from(path_buf),
        None => match PathBuf::from_str("") {
            Ok(path) => path,
            Err(e) => return Err(format!("Could not create empty path: {}", e)),
        },
    };

    // get file name
    let mut final_file_name = format!("{} - {}", downloaded_song.artist, downloaded_song.title);

    // replaces characters that are not allowed in file names with spaces, for both windows and linux (as they should be the same on both)
    const INVALID_WINDOWS_FILENAME_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

    for invalid_char in INVALID_WINDOWS_FILENAME_CHARS {
        final_file_name = final_file_name.replace(invalid_char, " ");
    }

    let renamed_file_path = renamed_file_path.join(format!(
        "downloaded/{}.mp3",
        final_file_name
    ));

    match std::fs::rename(&downloaded_song.file_location, &renamed_file_path) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!(
                "Could not rename {} to {}: {}",
                downloaded_song.file_location.to_string_lossy(),
                renamed_file_path.to_string_lossy(),
                e
            ))
        }
    };

    return Ok(renamed_file_path);
}
