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
    let renamed_file_path = renamed_file_path.join(format!(
        "downloaded/{} - {}.mp3",
        downloaded_song.artist, downloaded_song.title
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

// ensure file env exists and move all downloaded songs into the archive folder
pub fn init_file_env() -> Result<(), String> {
    // get working directory
    let working_directory = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("Could not get working directory: {}", e));
        }
    };

    // check if the archive directory exists, create it if it doesn't
    let downloaded_directory = working_directory.join("downloaded");

    // ensure that the downloaded directory exists
    if !downloaded_directory.exists() {
        if let Err(e) = std::fs::create_dir(&downloaded_directory) {
            return Err(format!("Could not create downloaded directory: {}", e));
        }
    }

    // check if the archive directory exists, create it if it doesn't
    let archive_directory = working_directory.join("archive");

    // ensure that the downloaded directory exists
    if !archive_directory.exists() {
        if let Err(e) = std::fs::create_dir(&archive_directory) {
            return Err(format!("Could not create archive directory: {}", e));
        }
    }

    move_downloaded_songs_to_archive()?;

    return Ok(());
}

pub fn move_downloaded_songs_to_archive() -> Result<(), String> {
    // get working directory
    let working_directory = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("Could not get working directory: {}", e));
        }
    };

    // ensure all songs in to_downloaded are now in the archive folder
    let downloaded_files = match std::fs::read_dir(working_directory.join("downloaded")) {
        Ok(files) => files,
        Err(e) => {
            return Err(format!(
                "Could not read files from downloaded directory: {}",
                e
            ))
        }
    };

    // assuming that every file in the downloaded directory is a music file
    for file_result in downloaded_files.into_iter() {
        let file = match file_result {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "Could not get file from file result in list downloaded files: {}",
                    e
                ))
            }
        };

        // move file into archive directory
        let from_path = file.path();
        let to_path = working_directory.join("archive").join(file.file_name());
        match std::fs::copy(from_path, to_path) {
            Ok(d) => d,
            Err(e) => {
                return Err(format!(
                    "Could not copy file {} from downloaded folder to archive folder: {}",
                    file.file_name().to_string_lossy(),
                    e
                ))
            }
        };
    }

    return Ok(());
}
