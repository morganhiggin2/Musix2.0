use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::{MusicSource, SongInformation};
use crate::yt_dlp_caller;

const LOCAL_PLAYLISTS_DIRECTORY: &str = "local_playlists";
const LOCAL_PLAYLIST_COLUMN_COUNT: usize = 4;

pub struct LocalPlaylistService {}

impl LocalPlaylistService {
    pub fn new() -> Self {
        return LocalPlaylistService {};
    }
}

impl MusicSource for LocalPlaylistService {
    fn download_song(
        &self,
        song_information: &SongInformation,
    ) -> Result<super::DownloadedSong, String> {
        return yt_dlp_caller::download_song(song_information);
    }

    fn get_playlist_song_information(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<super::SongInformation>, String> {
        // derive the local playlist file path from the playlist id
        let playlist_file_path = get_playlist_file_path(playlist_id)?;

        // parse the song information from the local playlist file
        let song_informations = read_playlist_song_informations(&playlist_file_path)?;

        return Ok(song_informations);
    }
}

// Ensure the local playlist file exists for the given playlist id
pub fn validate_local_playlist(playlist_id: &str) -> Result<(), String> {
    // derive the local playlist file path from the playlist id
    let playlist_file_path = get_playlist_file_path(playlist_id)?;

    // ensure the local playlist file exists
    if !Path::new(&playlist_file_path).exists() {
        return Err(format!(
            "Local playlist file does not exist: {}",
            playlist_file_path
        ));
    }

    return Ok(());
}

// Convert a local playlist id into the path of its local playlist file
fn get_playlist_file_path(playlist_id: &str) -> Result<String, String> {
    // strip the leading forward slash from the playlist id
    let playlist_file_name = match playlist_id.strip_prefix('/') {
        Some(file_name) => file_name,
        None => {
            return Err(format!(
                "Local playlist id must start with a forward slash: {}",
                playlist_id
            ));
        }
    };

    // ensure the playlist id has a file name
    if playlist_file_name.is_empty() {
        return Err(format!(
            "Local playlist id cannot be just a forward slash: {}",
            playlist_id
        ));
    }

    return Ok(format!(
        "{}/{}.txt",
        LOCAL_PLAYLISTS_DIRECTORY, playlist_file_name
    ));
}

// Read the song information from a local playlist file
fn read_playlist_song_informations(
    playlist_file_path: &str,
) -> Result<Vec<SongInformation>, String> {
    // ensure the local playlist file exists
    let playlist_file_path_buf = Path::new(playlist_file_path);
    if !playlist_file_path_buf.exists() {
        return Err(format!(
            "Local playlist file does not exist: {}",
            playlist_file_path
        ));
    }

    // open the local playlist file
    let playlist_file = match fs::File::open(playlist_file_path_buf) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!(
                "Could not open local playlist file {}: {}",
                playlist_file_path, e
            ));
        }
    };

    // read the header line, then each remaining line as song information
    let reader = BufReader::new(playlist_file);
    let mut song_informations = Vec::<SongInformation>::new();
    let mut is_first_line = true;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                return Err(format!(
                    "Could not read line from local playlist file {}: {}",
                    playlist_file_path, e
                ));
            }
        };

        // skip the header line
        if is_first_line {
            is_first_line = false;
            continue;
        }

        // skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // parse the song information from the line
        let song_information = parse_song_information_from_line(&line)?;

        song_informations.push(song_information);
    }

    return Ok(song_informations);
}

// Split a comma separated line into trimmed columns, honoring double quoted fields
fn split_line_into_columns(line: &str) -> Result<Vec<String>, String> {
    let mut columns = Vec::<String>::new();
    let mut current_column = String::new();
    let mut characters = line.chars().peekable();
    let mut is_in_quotes = false;

    while let Some(character) = characters.next() {
        match character {
            // a doubled quote inside a quoted field represents a literal quote
            '"' if is_in_quotes => match characters.peek() {
                Some('"') => {
                    characters.next();
                    current_column.push('"');
                }
                _ => is_in_quotes = false,
            },

            // only a quote at the start of a field opens a quoted field
            '"' if current_column.trim().is_empty() => {
                is_in_quotes = true;
                current_column.clear();
            }

            ',' if !is_in_quotes => {
                columns.push(current_column.trim().to_owned());
                current_column = String::new();
            }

            _ => current_column.push(character),
        }
    }

    // an unclosed quote means the line is malformed
    if is_in_quotes {
        return Err(format!(
            "Local playlist line has an unterminated quoted field: {}",
            line
        ));
    }

    columns.push(current_column.trim().to_owned());

    return Ok(columns);
}

// Parse a comma separated line into song information
fn parse_song_information_from_line(line: &str) -> Result<SongInformation, String> {
    // split the line into its columns, honoring quoted fields
    let columns = split_line_into_columns(line)?;

    // ensure the line has at least the url, artist, title, and genre columns
    if columns.len() < LOCAL_PLAYLIST_COLUMN_COUNT {
        return Err(format!(
            "Local playlist line does not have {} comma separated columns: {}",
            LOCAL_PLAYLIST_COLUMN_COUNT, line
        ));
    }

    // the url, artist, and genre are the first, second, and last columns
    let url = columns[0].clone();
    let artist = columns[1].clone();
    let genre = columns[columns.len() - 1].clone();

    // join the remaining columns back into the title to tolerate commas inside titles
    let title = columns[2..columns.len() - 1].join(", ").trim().to_owned();

    return Ok(SongInformation {
        url: url,
        title: title,
        genre: genre,
        artist: artist,
    });
}

#[cfg(test)]
mod tests {
    use super::parse_song_information_from_line;

    #[test]
    fn test_parse_song_information_from_line() {
        let song_information =
            parse_song_information_from_line("https://www.youtube.com/watch?v=OHJSX_MNpU4,MUST DIE!,Fever Dream (Part 1),Dance")
                .unwrap();

        assert_eq!(song_information.url, "https://www.youtube.com/watch?v=OHJSX_MNpU4");
        assert_eq!(song_information.artist, "MUST DIE!");
        assert_eq!(song_information.title, "Fever Dream (Part 1)");
        assert_eq!(song_information.genre, "Dance");

        let song_information = parse_song_information_from_line(
            "https://www.youtube.com/watch?v=j1n_8AuJwo4,\"Steve Aoki, Chris Lake & Tujamo\",Delirious (Boneless) [feat. Kid Ink],Dance",
        )
        .unwrap();

        assert_eq!(song_information.artist, "Steve Aoki, Chris Lake & Tujamo");
        assert_eq!(song_information.title, "Delirious (Boneless) [feat. Kid Ink]");
        assert_eq!(song_information.genre, "Dance");

        let song_information = parse_song_information_from_line(
            "https://www.youtube.com/watch?v=SyiVVjn1LIU,Todd Edwards Presents the Messenger,\"Main Vibe (12\"\" Extended Version)\",Electronic",
        )
        .unwrap();

        assert_eq!(song_information.artist, "Todd Edwards Presents the Messenger");
        assert_eq!(song_information.title, "Main Vibe (12\" Extended Version)");
        assert_eq!(song_information.genre, "Electronic");
    }

    #[test]
    fn test_parse_song_information_from_line_folds_extra_columns_into_title() {
        let song_information = parse_song_information_from_line(
            "https://www.youtube.com/watch?v=example,Artist,Track, With, A, Comma,Dance",
        )
        .unwrap();

        assert_eq!(song_information.artist, "Artist");
        assert_eq!(song_information.title, "Track, With, A, Comma");
        assert_eq!(song_information.genre, "Dance");
    }

    #[test]
    fn test_parse_song_information_from_line_rejects_short_lines() {
        let result = parse_song_information_from_line("https://www.youtube.com/watch?v=example,Artist,Track");

        assert!(result.is_err());
    }
}
