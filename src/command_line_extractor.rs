use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use clap::{Args, Parser, Subcommand};
use rustube::Video;

use crate::{
    audio_extractor::{EmptyAudioExtractor, FinishedAudioExtractor, InitializedAudioExtractor},
    audio_tag_appender::{
        EmptyAudioTagAppender, FinalizedAudioTagAppender, InitializedAudioTagAppender,
    },
    database::{self, Database, InitializedDatabase, UninitializedDatabase},
    title_extractor::{EmptyTitleExtractor, FinishedTitleExtractor, InitializedTitleExtractor},
    youtube_playlist_extractor::get_playlist_videos,
};

//TODO implement action for create playlist, including genre
//TODO implement actions to delete playlist
//TODO action to list playlists
//TODO action to run process

#[derive(Debug, Parser)]
pub struct App {
    #[clap(subcommand)]
    command: Command,
}

//TODO add help for each one

#[derive(Debug, Subcommand)]
pub enum Command {
    CreatePlaylist(CreatePlaylistArguments),
    DeletePlaylist(DeletePlaylistArguments),
    ListPlaylists,
    Run,
}

#[derive(Debug, Args)]
pub struct CreatePlaylistArguments {
    playlist_id: String,
    genre: String,
}

#[derive(Debug, Args)]
pub struct DeletePlaylistArguments {
    playlist_id: String,
}

pub fn parse_args(database_context: &mut Database) -> Result<(), String> {
    let args = App::parse();

    match args.command {
        Command::CreatePlaylist(args) => handle_create_playlist(args, database_context)?,
        Command::DeletePlaylist(args) => handle_delete_playlist(args, database_context)?,
        Command::ListPlaylists => handle_list_playlists(database_context)?,
        Command::Run => handle_run(database_context)?,
    }

    return Ok(());
}

/// Create playlist with name and id
pub fn handle_create_playlist(
    args: CreatePlaylistArguments,
    database_context: &mut Database,
) -> Result<(), String> {
    return database_context.put_playlist(args.playlist_id, args.genre);
}

/// Delete playlist by name
pub fn handle_delete_playlist(
    args: DeletePlaylistArguments,
    database_context: &mut Database,
) -> Result<(), String> {
    return database_context.delete_playlist(args.playlist_id);
}

/// List all playlists
pub fn handle_list_playlists(database_context: &mut Database) -> Result<(), String> {
    let playlists = database_context.get_all_playlists()?;

    // Print the playlists to the console
    // Currently just the id

    println!("Playlists: ");

    for playlist in playlists {
        println!("Playlist with id {}", playlist)
    }

    return Ok(());
}

// Handle run, which will attempt to download all the undownloaded videos from all the playlists in the database
pub async fn handle_run(database_context: &mut Database) -> Result<(), String> {
    // get list of playlists
    let playlist_ids = database_context.get_all_playlists()?;

    let mut all_video_ids = HashSet::<String>::new();

    for playlist_id in playlist_ids.to_owned() {
        // get videos
        let playlist_video_ids = get_playlist_videos(playlist_id).await?;

        // add to all videos
        playlist_video_ids.iter().for_each(|video_id| {
            all_video_ids.insert(video_id.video_id.to_owned());
        });
    }

    let mut downloaded_video_ids = HashSet::<String>::new();

    for playlist_id in playlist_ids.to_owned() {
        // get downloaded video ids
        let downloaded_playlist_video_ids = database_context.get_downloaded_videos(playlist_id)?;

        downloaded_playlist_video_ids.iter().for_each(|video_id| {
            downloaded_video_ids.insert(video_id.to_owned());
        });
    }

    // find vector of new videos
    let to_download_video_ids = all_video_ids.difference(&downloaded_video_ids);

    // download videos
    for to_download_video_id in to_download_video_ids {
        // create audio extractor
        let audio_extractor: InitializedAudioExtractor =
            EmptyAudioExtractor::init(to_download_video_id);
        let audio_extractor: FinishedAudioExtractor = audio_extractor.download()?;

        // get title from downloaded audio
        let title_extractor: InitializedTitleExtractor =
            EmptyTitleExtractor::init(audio_extractor.title().clone());
        let title_extractor: FinishedTitleExtractor = title_extractor.extract_from_title()?;

        println!(
            "song is at {} with title {}, name {}, and artist {} by video author {}",
            audio_extractor.write_path().as_os_str().to_str().unwrap(),
            audio_extractor.title(),
            title_extractor.name(),
            title_extractor.artist(),
            audio_extractor.author()
        );

        // append the tags to the video
        let tag_appender: InitializedAudioTagAppender =
            EmptyAudioTagAppender::init(&audio_extractor);
        let tag_appender: FinalizedAudioTagAppender = tag_appender.append_metadata()?;
        // get list of new videos
    }

    // for each new video
    //  download
    /*
    for playlist_id in playlists {
        // get all videos in playlist
        let playlist_video_ids = get_video_links(&playlist_id)?;

        println!("{}", playlist_video_ids.len());
    } */

    todo!();
}
