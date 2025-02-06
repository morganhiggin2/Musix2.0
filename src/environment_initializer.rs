use std::{io::Write};

use ureq;

// ensure the yt-dlp binary exists and is downloaded
pub fn init_yt_dlp_executable() -> Result<(), String> {
    let target_os = std::env::consts::OS;

    let yt_dlp_url = match target_os {
        "windows" => "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        "macos" => "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos",
        _ => {
            println!("OS is not windows or mac, assuming a Linux distro");
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
        }
    };
    let file_name = match target_os {
        "windows" => "yt-dlp.exe",
        _ => "yt-dlp",
    };

    // get working directory
    let working_directory = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("Could not get working directory: {}", e));
        }
    };
    let executable_path = working_directory.join(file_name);

    // if ytl-dlp.* exists, exit
    let executable_exists = match std::fs::exists(&executable_path) {
        Ok(val) => val,
        Err(e) => {
            return Err(format!(
                "Could not check if yt-dlp executable exists: {}",
                e
            ))
        }
    };
    if executable_exists {
        return Ok(());
    }

    // create executable file
    let mut file = match std::fs::File::create(&executable_path) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!(
                "Could not create file for yt-dlp executable: {}",
                e
            ))
        }
    };

    // download file using chuncking
    let request = ureq::get(yt_dlp_url).set("Transfer-Encoding", "chunked");
    let response = match request.call() {
        Ok(resp) => resp,
        Err(e) => {
            return Err(format!(
                "Error making request to download yt_dlp executable: {}",
                e
            ))
        }
    };
    let mut response_reader = response.into_reader();

    // read bytes from body reader into file
    let mut buffer = [0u8; 4048];

    loop {
        // read into buffer
        let bytes_read = match response_reader.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                return Err(format!(
                    "Could not read bytes in buffered read to download yt-dlp executable: {}",
                    e
                ))
            }
        };

        // we are done reading file
        if bytes_read == 0 {
            break;
        }

        // write but buffer contents to the file until all of them are written
        // in case there are partial writes
        let mut total_bytes_written = 0;
        // prevent infinite loop
        let mut tries = 0;

        loop {
            // write buffer to file
            let bytes_written = match file.write(&buffer[total_bytes_written..bytes_read]) {
                Ok(n) => n,
                Err(e) => {
                    return Err(format!(
                        "Could not write bytes in buffered write to download yt-dlp executable: {}",
                        e
                    ))
                }
            };

            total_bytes_written += bytes_written;

            if total_bytes_written == bytes_read {
                break;
            }

            if tries == 10 {
                return Err("Could not write enough bytes after 10 failed attempts".to_string());
            }

            tries += 1;
        }

        // clear buffer
        buffer.fill(0);
    }


    set_file_permissions(&mut file)?;

    return Ok(());
}


#[cfg(target_os = "windows")]
fn set_file_permissions(file: &mut std::fs::File) -> Result<(), String> {
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => return Err(format!("Could not get metadata from executable file: {}", e))
    };
    let mut permissions = metadata.permissions();

    // Set the file as read-only (Windows does not support Unix-style permissions)
    permissions.set_readonly(true);

    match file.set_permissions(permissions) {
        Ok(()) => (),
        Err(e) => {
            return Err(format!(
                "Could not set permissions for yt-dlp executable: {}",
                e
            ))
        }
    };
    
    return Ok(());
}

#[cfg(not(target_os = "windows"))]
fn set_file_permissions(file: &std::fs::File) -> Result<(), String>  {
    use std::os::unix::fs::PermissionsExt;

    match file.set_permissions(std::os::fs::Permissions::from_mode(0o755)) {
        Ok(()) => (),
        Err(e) => {
            return Err(format!(
                "Could not set permissions for yt-dlp executable: {}",
                e
            ))
        }
    };
    
    return Ok(());
}

// ensure file env exists and move all downloaded songs into the archive folder
pub fn init_file_env() -> Result<(), String> {
    // get working directory
    let working_directory = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("Could not get working directory: {}", e));
        }
    };

    // check if the downloaded directory exists, create it if it doesn't
    let data_directory = working_directory.join("data");

    // ensure that the downloaded directory exists
    if !data_directory.exists() {
        if let Err(e) = std::fs::create_dir(&data_directory) {
            return Err(format!("Could not create data directory: {}", e));
        }
    }
    
    // check if the downloaded directory exists, create it if it doesn't
    let database_directory = data_directory.join("database");

    // ensure that the downloaded directory exists
    if !database_directory.exists() {
        if let Err(e) = std::fs::create_dir(&database_directory) {
            return Err(format!("Could not create database directory: {}", e));
        }
    }

    // check if the downloaded directory exists, create it if it doesn't
    let downloaded_directory = working_directory.join("downloaded");

    // ensure that the downloaded directory exists
    if !downloaded_directory.exists() {
        if let Err(e) = std::fs::create_dir(&downloaded_directory) {
            return Err(format!("Could not create downloaded directory: {}", e));
        }
    }

    // check if the archive directory exists, create it if it doesn't
    let archive_directory = working_directory.join("archive");

    // ensure that the downloaded directory exists
    if !archive_directory.exists() {
        if let Err(e) = std::fs::create_dir(&archive_directory) {
            return Err(format!("Could not create archive directory: {}", e));
        }
    }

    move_downloaded_songs_to_archive()?;

    return Ok(());
}

pub fn move_downloaded_songs_to_archive() -> Result<(), String> {
    // get working directory
    let working_directory = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("Could not get working directory: {}", e));
        }
    };

    // ensure all songs in to_downloaded are now in the archive folder
    let downloaded_files = match std::fs::read_dir(working_directory.join("downloaded")) {
        Ok(files) => files,
        Err(e) => {
            return Err(format!(
                "Could not read files from downloaded directory: {}",
                e
            ))
        }
    };

    // assuming that every file in the downloaded directory is a music file
    for file_result in downloaded_files.into_iter() {
        let file = match file_result {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "Could not get file from file result in list downloaded files: {}",
                    e
                ))
            }
        };

        // move file into archive directory
        let from_path = file.path();
        let to_path = working_directory.join("archive").join(file.file_name());
        match std::fs::copy(from_path, to_path) {
            Ok(d) => d,
            Err(e) => {
                return Err(format!(
                    "Could not copy file {} from downloaded folder to archive folder: {}",
                    file.file_name().to_string_lossy(),
                    e
                ))
            }
        };
    }

    return Ok(());
}
