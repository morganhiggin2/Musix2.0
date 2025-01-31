use clap::{Args, Parser, Subcommand};
use rand::{self, Rng};
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use crate::{
    database::Database,
    environment_extractor::EnvironmentVariables,
    music_sources::{
        get_music_source_from_enum, get_music_source_from_url,
        soundcloud_service::SoundcloudMusicService, youtube_service::YoutubeMusicService,
        DownloadedSong, MusicSource,
    },
    post_processor,
};

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

pub fn parse_args(
    database_context: &mut Database,
    environment_variables: &EnvironmentVariables,
) -> Result<(), String> {
    let args = App::parse();

    match args.command {
        Command::CreatePlaylist(args) => {
            handle_create_playlist(args, database_context, environment_variables)?
        }
        Command::DeletePlaylist(args) => {
            handle_delete_playlist(args, database_context, environment_variables)?
        }
        Command::ListPlaylists => handle_list_playlists(database_context, environment_variables)?,
        Command::Run => handle_run(database_context, environment_variables)?,
    }

    return Ok(());
}

/// Create playlist with name and id
pub fn handle_create_playlist(
    args: CreatePlaylistArguments,
    database_context: &mut Database,
    environment_variables: &EnvironmentVariables,
) -> Result<(), String> {
    return database_context.put_playlist(args.playlist_id, args.genre, environment_variables);
}

/// Delete playlist by name
pub fn handle_delete_playlist(
    args: DeletePlaylistArguments,
    database_context: &mut Database,
    environment_variables: &EnvironmentVariables,
) -> Result<(), String> {
    return database_context.delete_playlist(args.playlist_id, environment_variables);
}

/// List all playlists
pub fn handle_list_playlists(
    database_context: &mut Database,
    environment_variables: &EnvironmentVariables,
) -> Result<(), String> {
    let playlists = database_context.get_all_playlists(environment_variables)?;

    // Print the playlists to the console
    println!("Playlists: ");

    for playlist in playlists {
        println!("Playlist with url {}", playlist)
    }

    return Ok(());
}

// Handle run, which will attempt to download all the undownloaded songs from all the playlists in the database
pub fn handle_run(
    database_context: &mut Database,
    environment_variables: &EnvironmentVariables,
) -> Result<(), String> {
    // get list of playlists
    let playlists = database_context.get_all_playlists(environment_variables)?;

    // list of downloaded song urls
    let mut downloaded_song_urls = HashSet::<String>::new();

    // get already downloaded songs for each playlist
    for playlist_url in playlists.to_owned() {
        // get downloaded song ids
        let downloaded_playlist_song_urls = database_context
            .get_downloaded_songs_from_playlist(playlist_url, environment_variables)?;

        downloaded_playlist_song_urls.iter().for_each(|song_url| {
            downloaded_song_urls.insert(song_url.to_owned());
        });
    }

    // ensure file dir is setup
    post_processor::init_file_env()?;

    // move any current songs in downloaded folder from last possible session into
    // the archive folder
    post_processor::move_downloaded_songs_to_archive()?;

    // for every playlist, download the songs that are in the playlist
    // but are not downloaded
    // TODO genre
    for playlist_url in playlists.to_owned() {
        // get music source type
        // this is unique for each playlist as a playlist can only have one source type
        let music_source_type = get_music_source_from_url(&playlist_url)?;

        // Create the designated music source
        // TODO how can it return both but yet be a generic for one?????
        let music_source: Box<dyn MusicSource> = get_music_source_from_enum(music_source_type);

        // get songs
        let playlist_song_urls = music_source.get_playlist_song_information(&playlist_url)?;

        // for each song in playlist song ids
        for to_download_song in playlist_song_urls {
            let song_url = to_download_song.url.to_owned();

            // if song has already been downloaded
            if downloaded_song_urls.contains(&playlist_url) {
                // do not download song, continue
                continue;
            }

            // download song
            let downloaded_song_result = music_source.download_song(&song_url);
            let downloaded_song = match downloaded_song_result {
                Ok(song_info) => song_info,
                Err(e) => {
                    println!(
                        "Song {} failed to downloaded, marking as failed: {}",
                        song_url, e
                    );

                    // put download song information into databse
                    database_context.put_downloaded_song(
                        song_url.to_owned(),
                        playlist_url.to_owned(),
                        true,
                        environment_variables,
                    )?;

                    continue;
                }
            };

            // post process song
            post_processor::post_process_downloaded_song(downloaded_song)?;

            // put download song information into databse
            database_context.put_downloaded_song(
                song_url.to_owned(),
                playlist_url.to_owned(),
                false,
                environment_variables,
            )?;

            // random sleep so we don't give the music provider sneaky suspicions *__*
            let sleep_time = rand::rng().random_range(2..7);
            thread::sleep(Duration::from_secs(sleep_time));
        }
    }

    return Ok(());
}
