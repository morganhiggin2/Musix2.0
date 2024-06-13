use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    //try previous failed videos again
    try_failed: Option<bool> 
}

pub fn parse_args() -> bool {
    let cli = Cli::parse();

    //options
    let mut option_try_failed: bool = false;

    if let Some(try_failed) = cli.try_failed {
        option_try_failed = try_failed;
    }

    return Ok(option_try_failed);
}