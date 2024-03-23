use std::str::FromStr;
use ytextract::{playlist::Id, Playlist};
use futures::{executor, StreamExt};

pub fn get_video_links(playlist_id: &str) -> Result<Vec<String>, String> {
    //create ytextract client
    let client = ytextract::Client::new();

    //get playlist id for ytextract library 
    let playlist_id_obj: Id = match Id::from_str(playlist_id) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    //create playlist object from playlist url
    //call main thread executor to run playlist stream future
    let playlist: Playlist = match executor::block_on(client.playlist(playlist_id_obj)) {
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
}