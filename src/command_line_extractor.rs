use std::{ops::{Deref, DerefMut}, sync::Mutex};

use clap::{Args, Parser, Subcommand};

use crate::{database::{self, Database, InitializedDatabase, UninitializedDatabase}, youtube_playlist_extractor::get_video_links};

//TODO implement action for create playlist, including genre
//TODO implement actions to delete playlist
//TODO action to list playlists
//TODO action to run process

#[derive(Debug, Parser)]
pub struct App {
    #[clap(subcommand)]
    command: Command
}

//TODO add help for each one

#[derive(Debug, Subcommand)]
pub enum Command {
    CreatePlaylist(CreatePlaylistArguments),
    DeletePlaylist(DeletePlaylistArguments),
    ListPlaylists,
    Run
}

#[derive(Debug, Args)]
pub struct CreatePlaylistArguments {
    playlist_id: String,
    genre: String
}

#[derive(Debug, Args)]
pub struct DeletePlaylistArguments {
    playlist_id: String
}

pub fn parse_args(database_context: &mut Database) -> Result<(), String> {
    let args = App::parse();

    match args.command {
        Command::CreatePlaylist(args) => {
            handle_create_playlist(args, database_context)?
        }
        Command::DeletePlaylist(args) => {
            handle_delete_playlist(args, database_context)?    
        }
        Command::ListPlaylists => {
            handle_list_playlists(database_context)? 
        }
        Command::Run => {
            handle_run(database_context)?
        }
    }    

    return Ok(());
}

/// Create playlist with name and id
pub fn handle_create_playlist(args: CreatePlaylistArguments, database_context: &mut Database) -> Result<(), String> {
    return database_context.put_playlist(args.playlist_id, args.genre);
}

/// Delete playlist by name 
pub fn handle_delete_playlist(args: DeletePlaylistArguments, database_context: &mut Database) -> Result<(), String> {
    return database_context.delete_playlist(args.playlist_id);
}

/// List all playlists
pub fn handle_list_playlists(database_context: &mut Database) -> Result<(), String> {
    let playlists = database_context.get_all_playlists()?;
    
    // Print the playlists to the console
    // Currently just the id
    for playlist in playlists {
        println!("Playlist with id {}", playlist)
    }

    return Ok(());
}

// Handle run, which will attempt to download all the undownloaded videos from all the playlists in the database
pub fn handle_run(database_context: &mut Database) -> Result<(), String> {
    // get list of playlists
    let playlists = database_context.get_all_playlists()?;

    // for each playlist
    for playlist_id in playlists {
        // get all videos in playlist
        let playlist_video_ids = get_video_links(&playlist_id)?;

        println!("{}", playlist_video_ids.len());
    }

    todo!();
}
