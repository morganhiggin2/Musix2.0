use crate::settings_parser::{self, Settings};

pub fn process() -> Result<(), String> {
    
}


fn sync_playlists_with_database() -> Result<(), String> {
    //parse settings
    //pass possible error though chain of calls
    let parsed_settings: Settings = settings_parser::parse_settings().unwrap(); 

    //get playlists
    let playlists = parsed_settings.get_playlists();

    
}