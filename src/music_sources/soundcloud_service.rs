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

use super::{MusicSource, SongInformation};

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

        let mut song_informations = Vec::<super::SongInformation>::new();

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

        //let window_hydration_start_i = match response_body.find("<script>window.__sc_hydration") {
        let window_hydration_start_i = match response_body.find("<script>window.__sc_hydration = ")
        {
            Some(i) => i + "<script>window.__sc_hydration = ".len(),
            None => {
                return Err("Could not find start window_hydration_extract variable in get soundcloud playlist page".to_string());
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

        let window_hydration_end_i = match response_body_remaining_slice.find(";</script>") {
            Some(i) => i,
            None => {
                return Err("Could not find end window_hydration_extract variable in get soundcloud playlist page".to_string());
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
        let hydration_json: serde_json::Value =
            match serde_json::from_str(&window_hydration_contents) {
                Ok(value) => value,
                Err(e) => {
                    return Err(format!(
                    "Failed to parse hydration JSON response in get soundcloud playlist tracks: {}",
                    e
                ));
                }
            };

        let hydration_array = match hydration_json.as_array() {
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

            if hydration_key_value == "playlist" {
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
                            "Could not convert track information json to array in get soundcloud playlist tracks"
                                .to_string(),
                        );
                    }
                };

                for hydration_track in hydration_tracks {
                    let track_id = match hydration_track.get("id") {
                        Some(data) => data,
                        None => {
                            return Err("Could not id from a track information track track in get soundcloud playlist tracks".to_owned());
                        }
                    };

                    //TODO
                    // match get permalink_url
                    // Some(_), get track information from this json
                    // None, get track information from track information url

                    song_informations.push(song_information)
                }
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

        return Ok(song_informations);
    }
}

fn get_track_information_from_track_id(track_id: String) -> Result<super::SongInformation, String> {
    // ------------ start get track information
    // create soundcloud track id request
    let get_track_information_url = format!("https://api-v2.soundcloud.com/tracks?ids={track_id}&client_id={client_id}&app_version=1737385876&app_locale=en");
    println!("{get_track_information_url}");

    // Fetch the main page of the playlist
    let track_information_response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(e) => return Err(format!("Error making get playlist request: {}", e)),
    };

    let track_information_response_body = match track_information_response.into_string() {
        Ok(text) => text,
        Err(e) => {
            return Err(format!(
                "Error retrieving response body from get playlist information request: {}",
                e
            ))
        }
    };

    let track_information_json: serde_json::Value = match serde_json::from_str(
        &track_information_response_body,
    ) {
        Ok(value) => value,
        Err(e) => {
            return Err(format!(
                                    "Failed to parse track information json response in get soundcloud playlist tracks: {}",
                                    e
                                ));
        }
    };

    let track_information_tracks = match track_information_json.as_array() {
        Some(data) => data,
        None => {
            // handle missing data here
            return Err(
                                    "Could not convert track information tracks to array in get soundcloud playlist tracks"
                                        .to_string(),
                                );
        }
    };

    let track_information_track = match track_information_json.get(0) {
        Some(data) => data,
        None => {
            return Err(
                "No tracks found in track information tracks in get soundcloud playlist tracks",
            )
            .to_string();
        }
    };
    // ------------ end get track information
}

fn get_song_information_from_track_information(
    information_json: &serde::Value,
) -> Result<SongInformation, String> {
    // permalink_url: url of the song to use
    let permalink_url = match information_json.get("permalink_url") {
        Some(data) => data,
        None => {
            // permalink could not be found, go and get the track information from the other links since it is going to be pagnated
            return Err(
                "Could not get permalink from a track information jsonin get soundcloud playlist tracks"
                    .to_owned(),
            );
        }
    };

    // genre: genre
    let genre = match information_json.get("genre") {
        Some(data) => data,
        None => {
            return Err(
                "Could not get genre from a track information jsonin get soundcloud playlist tracks"
                    .to_owned(),
            );
        }
    };

    // title: title
    let title = match information_json.get("title") {
        Some(data) => data,
        None => {
            return Err(
                "Could not get title from a track information jsonin get soundcloud playlist tracks"
                    .to_owned(),
            );
        }
    };

    // user.username
    let user_data = match information_json.get("user") {
        Some(data) => data,
        None => {
            return Err(
                "Could not get user from a track information jsonin get soundcloud playlist tracks"
                    .to_owned(),
            );
        }
    };

    // create downloadable song object
    let username = match user_data.get("username") {
        Some(data) => data,
        None => {
            return Err(
                "Could not get username from a track information jsonin get soundcloud playlist tracks"
                    .to_owned(),
            );
        }
    };

    // add to list of downloadable music
    let song_information = SongInformation {
        url: permalink_url.to_string(),
        title: title.to_string(),
        genre: genre.to_string(),
        artist: username.to_string(),
    };

    return Ok(song_information);
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
