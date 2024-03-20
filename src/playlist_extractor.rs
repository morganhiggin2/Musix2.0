use std::{future::IntoFuture, str::FromStr};

use ytextract::{playlist::{self, Id}, Playlist}

pub fn get_video_links(playlist_id: &str) -> Result<Vec<String>, String> {
    //TODO use futures_core executor 
    //create ytextract client
    let client = ytextract::Client::new();

    let playlist_id_obj: Id = match Id::from_str(playlist_id) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    //create playlist object from playlist url
    let playlist: Playlist = client.playlist(playlist_id_obj).wait(); 

    let video_ids: Vec<String> = Vec::new();

    //get urls from video
    for video in playlist.videos(){
        video_ids.append(video.id);
    }

    return Ok(video_ids);
}