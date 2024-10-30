// Rust code using external crates

use reqwest;
use serde_json::{json, Value};

const GOOGLE_API_KEY: &str = include_str!("resources/api_key.txt");

pub struct Video {
    // Define Video struct fields her
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub published_at: String,
}

pub async fn get_playlist_videos(playlist_id: String) -> Result<Vec<Video>, String> {
    let mut playlist_videos = Vec::new();
    let base_url = format!("https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&maxResults=25&playlistId={}&key={}&page_token=", playlist_id, GOOGLE_API_KEY);

    let mut page_token = "";
    let mut page_json: serde_json::Value;

    // create reqwest client
    let client = reqwest::Client::new();

    // for each page in the pagnated result
    loop {
        // get the next page
        let response = match client
            .get(format!("{}{}", base_url, page_token))
            .send()
            .await
        {
            Ok(some) => some,
            Err(e) => {
                return Err(format!(
                    "Could not make request to google api: {}",
                    e.to_string()
                ));
            }
        };

        let response_text = match response.text().await {
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
                    Some(video_id) => video_id.to_string(),
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
                Some(title) => title.to_string(),
                None => {
                    return Err(
                        "Could not get 'title' from 'snippet' in video information".to_string()
                    );
                }
            };

            let channel_title = match video_snippet.get("videoOwnerChannelTitle") {
                Some(channel_title) => channel_title.to_string(),
                None => {
                    return Err("Could not get 'videoOwnerChannelTitle' from 'snippet' in video information".to_string());
                }
            };

            let published_at = match video_snippet.get("publishedAt") {
                Some(value) => value.to_string(),
                None => {
                    return Err(
                        "Could not get 'publishedAt' from 'snippet' in video information"
                            .to_string(),
                    );
                }
            };

            // create Video instance with extracted data
            let new_video = Video {
                video_id,
                title,
                channel_title,
                published_at,
            };

            playlist_videos.push(new_video);
        }
    }

    return Ok(playlist_videos);
}
