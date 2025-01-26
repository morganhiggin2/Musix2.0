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
        let response = match ureq::get(url).call() {
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
            match regex::Regex::new("crossorigin src=\"(https:\\/[a-z0-9\\/\\-.]+\\.js)") {
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

        // we need to scrape the client id from the response body from each script
        // so get the response body for each script url and scrape it for the client id
        let client_id_regex = match regex::Regex::new("client_id=([A-z0-9]+)") {
            Ok(exp) => exp,
            Err(e) => {
                return Err(format!(
                    "Error creating client_id_regex Regex for playlist script scraping: {}",
                    e
                ))
            }
        };

        // collect all client id scape results for all the script files
        // Note: doing it this way to easily parallelize this in the short future
        let mut scripts_found_client_ids = Vec::<String>::new();

        // for each script
        for script_url in script_urls {
            // fetch script content
            // Fetch the main page of the playlist
            let response = match ureq::get(&script_url).call() {
                Ok(response) => response,
                Err(e) => {
                    return Err(format!(
                        "Error making get cross origin playlist script request: {}",
                        e
                    ))
                }
            };

            let response_body = match response.into_string() {
                Ok(text) => text,
                Err(e) => {
                    return Err(format!(
                        "Error retrieving response body from get cross origin playlist script request: {}",
                        e
                    ))
                }
            };

            // attempt to get the first match
            let script_url_scrape_matches = client_id_regex.captures(&response_body);

            match script_url_scrape_matches {
                // if a match
                Some(capture) => {
                    // add it to the list of found matches
                    let (_, [client_id]) = capture.extract();
                    scripts_found_client_ids.push(client_id.to_owned());
                }
                None => (),
            }
        }

        // because all the found client ids should be the same, get the first one
        let client_id = match scripts_found_client_ids.get(0) {
            Some(ele) => ele,
            None => {
                return Err(format!("Could not find client id in any of the cross origin scripts in getting soundcloud playlist tracks"))
            }
        };

        // using the original call's resposne body, get the inner window.__sc_hydration value
        let window_hydration_extract_regex =
            regex::Regex::new("\\<script\\>window\\.__sc_hydration[ ]*=[ ]*([.*]);\\<script\\>");

        let window_hydration_start_i = match response_body.find("<script>window.__sc_hydration") {
            Some(i) => i,
            None => {
                return Err("Could not find window_hydration_extract variable in get soundcloud playlist page".to_string());
            }
        };

        // find first occurance after previous index
        // TODO use iterator? first one gets the index
        let (_, response_body_remaining_slice) =
            match response_body.split_at_checked(window_hydration_start_i) {
                Some(slices) => slices,
                None => {
                    return Err(
                    "No remaining string in response body of main get soundcloud playlists request"
                        .to_string()
                    );
                }
            };

        let window_hydration_end_i = match response_body_remaining_slice.find(";<script>") {
            Some(i) => i + window_hydration_start_i,
            None => {
                return Err("Could not find window_hydration_extract variable in get soundcloud playlist page".to_string());
            }
        };

        let (window_hydration_contents, _) =
            match response_body_remaining_slice.split_at_checked(window_hydration_end_i) {
                Some(slice) => slice,
                None => return Err(
                    "window hydration variable search invalid in soundcloud playlist list tracks"
                        .to_string(),
                ),
            };

        // prase json in free manner
        let page_json: serde_json::Value = match serde_json::from_str(&response_body) {
            Ok(value) => value,
            Err(e) => {
                return Err(format!(
                    "Failed to parse JSON response in get soundcloud playlist tracks: {}",
                    e
                ));
            }
        };

        let hydration_array = match page_json.as_array() {
            Some(arr) => arr,
            None => {
                return Err(format!(
                    "Could not parse hydration variable contents as array in get soundcloud playlist tracks: {}",
                    response_body
                ));
            }
        };

        for hydration_element in hydration_array {
            // get hydration key
            let hydration_key_value = match hydration_element.get("hydratable") {
                Some(val) => val,
                None => {
                    return Err("Failed to get hydration key value from hydration array in get soundcloud playlist tracks".to_string());
                }
            };

            if hydration_key_value == "playlists" {
                let hydration_data = match hydration_element.get("data") {
                    Some(data) => data,
                    None => {
                        // handle missing data here
                        return Err(
                            "No data found in hydration elementn in get soundcloud playlist tracks"
                                .to_string(),
                        );
                    }
                };

                let hydration_track_data = match hydration_data.get("tracks") {
                    Some(data) => data,
                    None => {
                        // handle missing data here
                        return Err(
                            "No tracks found in hydration data in get soundcloud playlist tracks"
                                .to_string(),
                        );
                    }
                };

                let hydration_tracks = match hydration_track_data.as_array() {
                    Some(data) => data,
                    None => {
                        // handle missing data here
                        return Err(
                            "Could not convert hydration tracks to array in get soundcloud playlist tracks"
                                .to_string(),
                        );
                    }
                };

                for hydration_track in hydration_tracks {
                    // permalink_url: url of the song to use
                    // genre: genre
                    // title: title
                    // user.username
                    // create downloadable song object
                    // add to list of downloadable music
                }

                // get("tracks")
                // per track,
            }
        }

        // get first occurance of <script>window\.__sc_hydration
        // then get first occurance of ;<script>
        // trim inbetween string

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
