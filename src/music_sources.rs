pub mod soundcloud_service;
pub mod youtube_service;

#[derive(PartialEq, Eq)]
pub enum MusicSources {
    SOUNDCLOUD,
    YOUTUBE,
}

pub struct DownloadedSong {
    url: String,
    title: String,
    genre: String,
    artist: String,
}

pub struct SongInformation {
    pub url: String,
    pub title: String,
    pub genre: String,
    pub artist: String,
}

/* Common trait defining the behavior of a music service */
pub trait MusicSource {
    fn download_song(&self, url: &str) -> Result<DownloadedSong, String>;
    fn get_playlist_song_information(&self, url: &str) -> Result<Vec<SongInformation>, String>;
}

pub fn get_music_source_from_url(url: &str) -> Result<MusicSources, String> {
    // get origin of the url
    let origin_regex = match regex::Regex::new(r"your_regex_here") {
        Ok(regex) => regex,
        Err(e) => {
            return Err(format!("Error in regex creation: {}", e));
        }
    };

    let mut regex_match_iter = origin_regex.captures_iter(&url);

    let regex_catpure_group = match regex_match_iter.next() {
        Some(capture) => capture,
        None => return Err(format!("Could not find origin url in url: {}", url)),
    };

    let (_full, [origin]) = regex_catpure_group.extract();

    // assert there is only one match
    assert!(
        regex_match_iter.next().is_none(),
        "Origin extraction from music source url was only supposed to have one origin"
    );

    let origin_enum = match origin {
        "soundcloud" => MusicSources::SOUNDCLOUD,
        "youtube" => MusicSources::YOUTUBE,
        _ => {
            return Err(format!(
                "Music url origin is not of the supported types: {}",
                origin
            ));
        }
    };

    return Ok(origin_enum);
}

pub fn get_music_source_from_enum(music_source: MusicSources) -> Box<dyn MusicSource> {
    if music_source == MusicSources::SOUNDCLOUD {
        return Box::new(soundcloud_service::SoundcloudMusicService::new());
    } else {
        return Box::new(youtube_service::YoutubeMusicService::new());
    }
}
