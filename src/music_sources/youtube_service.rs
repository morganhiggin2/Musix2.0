use ureq;

use super::{MusicSource, SongInformation};
use crate::title_extractor::{
    EmptyTitleExtractor, FinishedTitleExtractor, InitializedTitleExtractor,
};
use crate::yt_dlp_caller;

const GOOGLE_API_KEY: &str = include_str!("../resources/api_key.txt");

pub struct YoutubeMusicService {}

pub struct Video {
    // Define Video struct fields her
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub published_at: String,
}

impl YoutubeMusicService {
    pub fn new() -> Self {
        return YoutubeMusicService {};
    }
}

impl MusicSource for YoutubeMusicService {
    fn download_song(
        &self,
        song_information: &SongInformation,
    ) -> Result<super::DownloadedSong, String> {
        return yt_dlp_caller::download_song(&song_information);
    }

    fn get_playlist_song_information(
        &self,
        url: &str,
    ) -> Result<Vec<super::SongInformation>, String> {
        // get playlist id from url
        let url_regex =
            match regex::Regex::new(r"https://www\.youtube\.com/playlist\?list=([A-z0-9]+)") {
                Ok(regex) => regex,
                Err(err) => return Err(format!("Failed to create regex: {}", err)),
            };
        let playlist_url = url_regex
            .captures(url)
            .and_then(|caps| caps.get(1))
            .ok_or("Could not extract playlist id from url")?
            .as_str();

        // get playlist information from https://www.googleapis.com/youtube/v3/playlists?part=snippet%2Clocalizations&id=" + playlistId + "&fields=items(localizations%2Csnippet%2Flocalized%2Ftitle)&key=" + KEY;
        let url = format!("https://www.googleapis.com/youtube/v3/playlists?part=snippet%2Clocalizations&id={}&fields=items(localizations%2Csnippet%2Flocalized%2Ftitle)&key={}", playlist_url, GOOGLE_API_KEY);
        let response = match ureq::get(&url).call() {
            Ok(response) => response,
            Err(err) => {
                return Err(format!(
                    "Failed to make get playlist information request: {}",
                    err.to_string()
                ))
            }
        };
        let response_text = match response.into_string() {
            Ok(text) => text,
            Err(e) => return Err(format!("Failed to get response text: {}", e)),
        };

        // get page json
        let page_json: serde_json::Value = match serde_json::from_str(&response_text) {
            Ok(value) => value,
            Err(e) => {
                return Err(format!("Failed to parse JSON response: {}", e));
            }
        };

        // search page json for playlist title
        // items[0].snippet.localized.title
        let items_array = match page_json.get("items") {
            Some(items) => items.as_array().unwrap(),
            None => {
                return Err(format!("Could not extract 'items' array from page json"));
            }
        };
        let item = items_array
            .get(0)
            .ok_or("Could not get 0th item from items array in page json")?;

        let snippet_information = match item.get("snippet") {
            Some(item) => item,
            None => {
                return Err(format!(
                    "Could not extract snippet information from item element in page json"
                ));
            }
        };

        let localized_information = match snippet_information.get("localized") {
            Some(item) => item,
            None => {
                return Err(format!(
                    "Could not extract localized information from snippet information in page json"
                ));
            }
        };

        let title_information = match localized_information.get("title") {
            Some(item) => item,
            None => {
                return Err(format!("Could not extract title information array from localized information in page json"));
            }
        };

        let playlist_title = title_information
            .as_str()
            .ok_or("Could not convert title information to string")?;

        let mut playlist_videos = Vec::new();
        let base_url = format!("https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&maxResults=25&playlistId={}&key={}&page_token=", playlist_url, GOOGLE_API_KEY);

        let mut page_token = "";
        let mut page_json: serde_json::Value;

        // for each page in the pagnated result
        loop {
            // get the next page
            let response = match ureq::get(&format!("{}{}", base_url, page_token)).call() {
                Ok(some) => some,
                Err(e) => {
                    return Err(format!(
                        "Could not make request to google api: {}",
                        e.to_string()
                    ));
                }
            };

            let response_text = match response.into_string() {
                Ok(text) => text,
                Err(e) => return Err(format!("Failed to get response text: {}", e)),
            };

            // get page json
            page_json = match serde_json::from_str(&response_text) {
                Ok(value) => value,
                Err(e) => {
                    return Err(format!("Failed to parse JSON response: {}", e));
                }
            };

            // if error exists
            match page_json.get("error") {
                Some(error) => {
                    return Err(format!(
                        "Error getting playlist information for playlist {}",
                        error
                    ));
                }
                None => (),
            };

            // get playlist video items
            let urls_array = match page_json.get("items") {
                Some(items) => items.as_array().unwrap(),
                None => {
                    return Err(format!("Could not extract 'items' array from page json"));
                }
            };

            // for video token in array
            for video_information in urls_array.iter() {
                // if this is marked as a private video
                let video_snippet = match video_information.get("snippet") {
                    Some(snippet) => snippet,
                    None => {
                        return Err("Could not get 'snippet' from video information".to_string());
                    }
                };

                match video_snippet.get("description") {
                    Some(description) => {
                        if description.eq("This video is private.") {
                            // skip video
                            break;
                        }
                    }
                    None => {
                        return Err(format!("Could not get 'description' in video information"));
                    }
                }

                // get video information
                let video_id = match video_snippet.get("resourceId") {
                    Some(resource_id) => match resource_id.get("videoId") {
                        Some(video_id) => video_id.as_str().unwrap_or(""),
                        None => {
                            return Err(
                                "Could not get 'videoId' from 'resourceId' in video information"
                                    .to_string(),
                            );
                        }
                    },
                    None => {
                        return Err(
                            "Could not get 'resourceId' from 'snippet' in video information"
                                .to_string(),
                        );
                    }
                };

                let title = match video_snippet.get("title").as_ref() {
                    Some(title) => title.as_str().unwrap_or(""),
                    None => {
                        return Err(
                            "Could not get 'title' from 'snippet' in video information".to_string()
                        );
                    }
                };

                let channel_title = match video_snippet.get("videoOwnerChannelTitle") {
                    Some(channel_title) => channel_title.as_str().unwrap_or(""),
                    None => {
                        return Err("Could not get 'videoOwnerChannelTitle' from 'snippet' in video information".to_string());
                    }
                };

                let _published_at = match video_snippet.get("publishedAt") {
                    Some(value) => value.as_str().unwrap_or(""),
                    None => {
                        return Err(
                            "Could not get 'publishedAt' from 'snippet' in video information"
                                .to_string(),
                        );
                    }
                };

                // get title from youtube video title
                let title_extractor: InitializedTitleExtractor =
                    EmptyTitleExtractor::init(title.to_string(), channel_title.to_string());
                let title_extractor: FinishedTitleExtractor =
                    title_extractor.extract_from_title()?;

                // create Video instance with extracted data
                let song_information = super::SongInformation {
                    url: url.to_owned(),
                    title: title_extractor.name().to_owned(),
                    // genre is the title of the playlist
                    genre: playlist_title.to_owned(),
                    artist: title_extractor.artist().to_owned(),
                };

                playlist_videos.push(song_information);
            }

            // get next page token
            page_token = match page_json.get("nextPageToken") {
                Some(next_page_token) => match next_page_token.as_str() {
                    Some(token) => token,
                    None => {
                        return Err(format!(
                            "Could not get next page token from found next page token in json"
                        ));
                    }
                },
                None => {
                    break;
                }
            };
        }

        return Ok(playlist_videos);
    }
}
