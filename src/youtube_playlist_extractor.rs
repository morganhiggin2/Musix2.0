/*use std::str::FromStr;
use rustube::tokio;
use ytextract::{playlist::Id, Playlist};
use futures::{executor, StreamExt};
use tokio::runtime::Runtime;

pub fn get_video_links(playlist_id: &str) -> Result<Vec<String>, String> {
    //create ytextract client
    let client = ytextract::Client::new();

    //create tokio runtime
    let tokio_rt = match Runtime::new() {
        Ok(some) => some,
        Err(e) => {
            return Err(format!("Unable to create tokio runtime in get video links method: {}", e));
        }
    };

    //get playlist id for ytextract library 
    let playlist_id_obj: Id = match Id::from_str(playlist_id) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    //create playlist object from playlist url
    //call main thread executor to run playlist stream future
    let playlist: Playlist = match tokio_rt.block_on(client.playlist(playlist_id_obj)) {
        Ok(some) => some,
        Err(e) => {
            return Err(e.to_string());
        }
    }; 

    let mut video_ids: Vec<String> = Vec::new();

    //Put playlist object in boxed, pinning it to the local thread
    let videos_stream = playlist.videos().boxed();
    let videos = executor::block_on_stream(videos_stream);

    //get urls from video
    for video_iter_element in videos.enumerate(){
        //Check if video information got successfully loaded
        let video = match video_iter_element.1 {
            Ok(some) => some,
            Err(e) => {
                //Error handling video, warn of corrupt video link and move along video_iter_element.0
                println!("Video with id {} is corrupt in playlist {}", video_iter_element.0, playlist_id);
                continue;
            }
        };

        video_ids.push(video.id().to_string());
    }

    return Ok(video_ids);
}*/






/*
//Credit to YT Extractor
use std::time::Duration;

use serde::Serialize;

use crate::{youtube::player_response, Error};

const RETRYS: u32 = 5;
const TIMEOUT: Duration = Duration::from_secs(30);
const DUMP: bool = option_env!("YTEXTRACT_DUMP").is_some();
const DUMP_ERR: bool = option_env!("YTEXTRACT_DUMP_ERR").is_some();
const BASE_URL: &str = "https://youtubei.googleapis.com/youtubei/v1";
const API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

const CONTEXT_WEB: Context<'static> = Context {
    client: Client {
        hl: "en",
        gl: "US",
        client_name: "WEB",
        client_version: "2.20210622.10.0",
    },
};

const CONTEXT_ANDROID: Context<'static> = Context {
    client: Client {
        hl: "en",
        gl: "US",
        client_name: "ANDROID",
        client_version: "16.05",
    },
};

const CONTEXT_EMBEDDED: Context<'static> = Context {
    client: Client {
        hl: "en",
        gl: "US",
        client_name: "TVHTML5_SIMPLY_EMBEDDED_PLAYER",
        client_version: "2.0",
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Context<'a> {
    client: Client<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Client<'a> {
    hl: &'a str,
    gl: &'a str,
    client_name: &'a str,
    client_version: &'a str,
}

pub enum ChannelPage {
    About,
}

pub enum Browse {
    Playlist(crate::playlist::Id),
    Channel {
        id: crate::channel::Id,
        page: ChannelPage,
    },
    Continuation(String),
}

pub enum Next {
    Video(crate::video::Id),
    Continuation(String),
}

#[derive(Clone, Default)]
pub struct Api {
    pub(crate) http: reqwest::Client,
}

fn dump(endpoint: &'static str, response: &str) {
    let _ = std::fs::create_dir(endpoint);
    use std::time::SystemTime;
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("TIME");
    std::fs::write(
        &format!("{}/{}.json", endpoint, time.as_millis()),
        &response,
    )
    .expect("Write");
}

impl Api {
    async fn get<T: serde::de::DeserializeOwned, R: Serialize + Send + Sync>(
        &self,
        endpoint: &'static str,
        request: R,
        context: Context<'static>,
    ) -> crate::Result<T> {
        #[derive(Serialize)]
        struct Request<R: Serialize> {
            context: Context<'static>,
            #[serde(flatten)]
            request: R,
        }

        let request = Request { context, request };

        let request = self
            .http
            .post(format!("{}/{}", BASE_URL, endpoint))
            .header("X-Goog-Api-Key", API_KEY)
            .json(&request)
            .timeout(TIMEOUT);

        let mut retry = 0;

        loop {
            let response = request
                .try_clone()
                .unwrap()
                .send()
                .await
                .and_then(|x| x.error_for_status())
                .map(|x| x.text());

            match response {
                Ok(res) => {
                    let response = res.await?;

                    let res = serde_json::from_str::<T>(&response);
                    if DUMP || (DUMP_ERR && res.is_err()) {
                        dump(endpoint, &response)
                    }
                    let res = res.expect("Failed to parse JSON");
                    break Ok(res);
                }
                Err(err) => {
                    if err.is_timeout() {
                        if retry == RETRYS {
                            log::error!("Timed out {} times. Stopping...", RETRYS);
                            break Err(Error::Request(err));
                        } else {
                            log::warn!("Timeout reached, retrying...");
                            retry += 1;
                            continue;
                        }
                    } else {
                        break Err(Error::Request(err));
                    }
                }
            }
        }
    }

    pub async fn streams(
        &self,
        id: crate::video::Id,
    ) -> crate::Result<player_response::StreamPlayerResponse> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            video_id: crate::video::Id,
        }

        let request = Request { video_id: id };
        let res = self
            .get("player", &request, CONTEXT_ANDROID)
            .await
            .and_then(
                |x: player_response::Result<player_response::StreamPlayerResponse>| x.into_std(),
            );

        // If this is a age-restricted error, retry with an embedded player
        if matches!(res, Err(crate::Error::Youtube(ref yt)) if yt.to_string().contains("age")) {
            self.get("player", request, CONTEXT_EMBEDDED)
                .await
                .and_then(
                    |x: player_response::Result<player_response::StreamPlayerResponse>| {
                        x.into_std()
                    },
                )
        } else {
            res
        }
    }

    pub async fn player(
        &self,
        id: crate::video::Id,
    ) -> crate::Result<player_response::Result<player_response::PlayerResponse>> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            video_id: crate::video::Id,
        }

        let request = Request { video_id: id };

        self.get("player", request, CONTEXT_ANDROID).await
    }

    pub async fn next<T: serde::de::DeserializeOwned>(&self, next: Next) -> crate::Result<T> {
        match next {
            Next::Video(video_id) => {
                #[derive(Debug, Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request {
                    video_id: crate::video::Id,
                }

                let request = Request { video_id };

                self.get("next", request, CONTEXT_WEB).await
            }
            Next::Continuation(continuation) => {
                #[derive(Debug, Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request {
                    continuation: String,
                }

                let request = Request { continuation };

                self.get("next", request, CONTEXT_WEB).await
            }
        }
    }

    pub async fn browse<T: serde::de::DeserializeOwned>(&self, browse: Browse) -> crate::Result<T> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            browse_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<String>,
        }

        let request = match browse {
            Browse::Playlist(id) => Request {
                browse_id: format!("VL{}", id),
                params: Some(base64::encode([0xc2, 0x06, 0x02, 0x08, 0x00])),
            },
            Browse::Channel { id, page } => Request {
                browse_id: format!("{}", id),
                params: match page {
                    ChannelPage::About => Some(base64::encode(b"\x12\x05about")),
                },
            },
            Browse::Continuation(continuation) => {
                #[derive(Debug, Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request {
                    continuation: String,
                }

                let request = Request { continuation };

                return self.get("browse", request, CONTEXT_WEB).await;
            }
        };

        self.get("browse", request, CONTEXT_WEB).await
    }
}*/

// google playlist api: build hub, then call playlist__ and build with playlist id, page id, etc..., then doit()
// y = youtube::new()
// p = y.playlist_items()
// p.list(part???)
/*
///              .video_id("nonumy")
///              .playlist_id("rebum.")
///              .page_token("tempor")
///              .on_behalf_of_content_owner("dolore")
///              .max_results(76)
///              .add_id("amet.")
///              .doit().await;*/