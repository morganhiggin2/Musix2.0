// Download songs to location (write location, soundcloud url)
//  download_sleep
//  call yt-dlp cli with url and write location
//  wait for command to finish executing, capturing any errors
//  retry if necessary, exponential backoff in sleep with gradual coming back to original sleep after successfull completion (via averaging)
//  after retry limit, continue with other songs, logged failed song
//
// return list of unsuccessful urls

// donwload songs from playlist

use regex;
use ureq;

use super::MusicSource;

pub struct SoundcloudMusicService {}

impl SoundcloudMusicService {
    pub fn new() -> Self {
        return SoundcloudMusicService {};
    }
}

/* implement the common behvaior for a music service */
impl MusicSource for SoundcloudMusicService {
    fn download_song(&self, url: &str) -> Result<super::DownloadedSong, String> {
        // format

        /*
        // build cli command
        let cli_command = format!(
            "yt-dlp {} --audio-format mp3 --username {} --password {}",
            url, username, password
        );*/

        todo!()
    }

    fn get_playlist_song_information(
        &self,
        url: &str,
    ) -> Result<Vec<super::SongInformation>, String> {
        // Fetch the main page of the playlist
        let response = match ureq::get("").call() {
            Ok(response) => response,
            Err(e) => return Err(format!("Error making get playlist request: {}", e)),
        };

        let response_body = match response.into_string() {
            Ok(text) => text,
            Err(e) => {
                return Err(format!(
                    "Error retrieving response body from get playlist information request: {}",
                    e
                ))
            }
        };

        // get the cross origin javascript scripts which are referenced in the file
        // that are normally hotloaded
        let script_url_scrape_regex =
            match regex::Regex::new("crossorigin src=\"(https:\\\\/[a-z0-9\\/\\-.]+\\.js)") {
                Ok(exp) => exp,
                Err(e) => {
                    return Err(format!(
                        "Error creating script_url_scrape_regex Regex: {}",
                        e
                    ))
                }
            };

        // get scripts listed in file with https://a-v2.sndcdn.com/assets/([A-z0-9-]+\.js)
        // extract the script files using regex
        let script_url_scrape_matches = script_url_scrape_regex.captures_iter(&response_body);

        let mut script_urls = Vec::<String>::new();

        for script_url_match in script_url_scrape_matches {
            let (_full, [url]) = script_url_match.extract();

            script_urls.push(url.to_owned());
        }

        // for each script
        // TODO parallelize, make futures, and wait all their executions
        for script_url in script_urls {
            // fetch script content
        }

        //   fetch scipt contents
        //   look for client_id=([A-z0-9]{32})
        // store client_id
        //
        // for the window.__sc_hydration variable, get the element with hydration="platlists", each element is a track with an id
        //   - parse each <script> to </script>, getting the window.... variable, then inside the = and last ;
        //      filter by motization???
        // for each track in main file
        //   call the tracks url to get the track information
        //   in the track id, get the first level permalink_url variable for the soundcloud url
        //      - get genre
        //      - get publisher_metadata.artist
        //      - get title

        return Ok(vec![]);
    }
}

/*
fn download_video(url: String, username: String, password: String) -> Result<(), String> {

}

struct SoundCloudPlaylistTrack {
}

pub async fn get_soundcloud_playlist_tracks(
    playlist_url: String,
) -> Result<Vec<SoundCloudPlaylistTrack>, String> {
    }
 */
