// Clean the url so that two urls with the same origins are equal
// ex. https://www.google.com == www.google.com
pub fn enforce_url(url: &str) -> Result<(), String> {
    let origin_regex = match regex::Regex::new(r"https:\/\/([A-z0-9_-]+)\.([A-z0-9_-]+){1}\.") {
        Ok(reg) => reg,
        Err(e) => {
            return (Err(format!(
                "Could not create origin regex in enforce url: {}",
                e
            )))
        }
    };

    match origin_regex.find(url) {
        Some(_) => return Ok(()),
        None => return(Err(format!("The url {} does not meet url stanrdards, ensure there exists the https protocol, and a sub origin, origin, and root origin in the url", url)))
    }
}
