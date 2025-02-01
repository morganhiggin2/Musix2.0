use std::process::Command;

use crate::music_sources::{DownloadedSong, SongInformation};

// Calls the yt-dlp cli via os commands
pub fn download_song(song_information: &SongInformation) -> Result<DownloadedSong, String> {
    let working_directory = match std::env::current_dir() {
        Ok(pwd) => pwd,
        Err(e) => {
            return Err(format!(
                "Could not get working directory in download song: {}",
                e
            ));
        }
    };

    let command_output = match Command::new("./yt-dlp")
        .current_dir(&working_directory)
        .arg("-o")
        .arg("tmp")
        .arg("--audio-format")
        .arg("mp3")
        .arg("-x")
        .arg(&song_information.url)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            return Err(format!(
                "Could not spawn process to download song from url {}: {}",
                song_information.url, e
            ));
        }
    };

    let download_song_path = working_directory.join("tmp.mp3");

    if command_output.status.success() {
        return Ok(DownloadedSong {
            url: song_information.url.to_owned(),
            title: song_information.title.to_owned(),
            genre: song_information.genre.to_owned(),
            artist: song_information.artist.to_owned(),
            file_location: download_song_path,
        });
    } else {
        let child_process_stderr_vec = command_output.stderr;

        let stderr_string = match String::from_utf8(child_process_stderr_vec) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Could not convert std out bytes buffer to utf8 string: {}",
                    e
                ));
            }
        };

        return Err(format!(
            "Could not download file, processes exited with stderr: {}",
            stderr_string
        ));
    }
}
