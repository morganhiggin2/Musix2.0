use clap::{Args, Parser, Subcommand};

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
    name: String,
    playlist_id: String 
}

#[derive(Debug, Args)]
pub struct DeletePlaylistArguments {
    name: String
}

pub fn parse_args() -> Result<(), String> {
    let args = App::parse();

    match args.command {
        Command::CreatePlaylist(args) => {
            handle_create_playlist(args)?
        }
        Command::DeletePlaylist(args) => {
            handle_delete_playlist(args)?    
        }
        Command::ListPlaylists => {
            todo!();
        }
        Command::Run => {
            todo!();
        }
    }    

    return Ok(());
}

/// Create playlist with name and id
pub fn handle_create_playlist(args: CreatePlaylistArguments) -> Result<(), String> {
    
    return Ok(());
}

/// Delete playlist by name 
pub fn handle_delete_playlist(args: DeletePlaylistArguments) -> Result<(), String> {

    return Ok(());
}

/*let foo = Command::new("foo")
        .description("Shows foo")
        .options(|app| {
            app.arg(
                Arg::with_name("debug")
                    .short("d")
                    .help("Prints debug information verbosely"),
            )
        })
        // Putting argument types here for clarity
        .runner(|args: &str, matches: &ArgMatches<'_>| {
            let debug = clap::value_t!(matches, "debug", bool).unwrap_or_default();
            println!("Running foo, env = {}, debug = {}", args, debug);
            Ok(())
        });

    let bar = Command::new("bar")
        .description("Shows bar")
        // Putting argument types here for clarity
        .runner(|args: &str, _matches: &ArgMatches<'_>| {
            println!("Running bar, env = {}", args);
            Ok(())
        });

    Commander::new()
        .options(|app| {
            app.arg(
                Arg::with_name("environment")
                    .short("e")
                    .long("env")
                    .global(true)
                    .takes_value(true)
                    .value_name("STRING")
                    .help("Sets an environment value, defaults to \"dev\""),
            )
        })
        // `Commander::args()` derives arguments to pass to subcommands.
        // Notice all subcommands (i.e. `foo` and `bar`) will accept `&str` as arguments.
        .args(|_args, matches| matches.value_of("environment").unwrap_or("dev"))
        // Add all subcommands
        .add_cmd(foo)
        .add_cmd(bar)
        // To handle when no subcommands match
        .no_cmd(|_args, _matches| {
            println!("No subcommand matched");
            Ok(())
        })
        .run(); */