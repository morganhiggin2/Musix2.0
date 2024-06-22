use crate::settings_parser::{self, Settings};

/// Responible for downloading all desired music based on the settings
pub fn process() -> Result<(), String> {
    todo!();
}

fn sync_playlists_with_database() -> Result<(), String> {
    //parse settings
    //pass possible error though chain of calls
    //TODO should be called at the very beginning of the program
    //let parsed_settings: Settings = settings_parser::parse_settings().unwrap(); 

    //get all playlists
    //let playlists = parsed_settings.get_all_playlists();

    //for each playlist
        //get list of vidoes in the current playlist in youtube

        //get list of videos in the database that have been read

        //from these, get list of videos which we need to download
        //also get list of videos that don't exist anymore, just create nice warning

    todo!();
    
}