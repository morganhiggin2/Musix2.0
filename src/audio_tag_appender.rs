use id3::TagLike;

use crate::music_sources;

pub fn append_metadata(song_information: &music_sources::DownloadedSong) -> Result<(), String> {
    // read metadata from existing file
    let mut current_tags = match id3::Tag::read_from_path(&song_information.file_location) {
        Ok(val) => val,
        Err(e) => {
            return Err(format!("Could not get tags from audio file: {}", e));
        }
    };

    // set appropriate tags
    current_tags.set_title(song_information.title.as_str());
    current_tags.set_artist(song_information.artist.as_str());
    current_tags.set_genre(song_information.genre.as_str());

    // write tags back through write_to_path, which replaces only the tag region;
    // writing the tag through an open file handle overwrites the start of the first
    // audio frame when the new tag is longer than the old one, destroying the xing
    // header players use to determine the song duration
    match current_tags.write_to_path(&song_information.file_location, id3::Version::Id3v24) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Cannot write to audio file: {}", e));
        }
    };

    return Ok(());
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::music_sources::DownloadedSong;
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn test_append_metadata_preserves_xing_duration_header() {
        // generate a small sample mp3 with ffmpeg, the same encoder the download pipeline uses
        let sample_path = std::env::temp_dir().join("musix_tag_appender_test.mp3");
        let ffmpeg_output = Command::new("ffmpeg")
            .args(["-v", "error", "-f", "lavfi", "-i", "sine=frequency=440:duration=2", "-codec:a", "libmp3lame", "-y"])
            .arg(&sample_path)
            .output()
            .expect("ffmpeg must be available to generate the test sample");
        assert!(ffmpeg_output.status.success(), "ffmpeg failed to generate the test sample mp3");

        let song_information = DownloadedSong {
            url: "https://www.youtube.com/watch?v=test".to_string(),
            title: "Test Title".to_string(),
            genre: "Test Genre".to_string(),
            artist: "Test Artist".to_string(),
            file_location: sample_path.clone(),
        };

        let result = append_metadata(&song_information);
        assert!(result.is_ok());

        // the xing/info duration header in the first audio frame must survive tagging,
        // otherwise players estimate the duration from the bitrate and report wrong times
        let tagged_bytes = std::fs::read(&sample_path).expect("could not read tagged sample");
        assert!(
            tagged_bytes.windows(4).any(|window| window == b"Xing" || window == b"Info"),
            "xing duration header was stripped by tagging"
        );

        // tags must read back with the written values
        let tags = id3::Tag::read_from_path(&sample_path).expect("could not read tags back");
        assert_eq!(tags.title(), Some("Test Title"));
        assert_eq!(tags.artist(), Some("Test Artist"));
        assert_eq!(tags.genre(), Some("Test Genre"));

        let _ = std::fs::remove_file(&sample_path);
    }
}
