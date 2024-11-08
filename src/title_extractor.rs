use getset::Getters;
use std::ops::Deref;

use regex::Regex;

const SPACE_REGEX: &str = r"([ ]{2,})";
const TITLE_SEPERATOR_REGEX: &str = r"[-]+";

pub struct EmptyTitleExtractor;

pub struct InitializedTitleExtractor {
    title: String,
}

#[derive(Getters)]
pub struct FinishedTitleExtractor {
    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    artist: String,
}

impl EmptyTitleExtractor {
    pub fn init(title: String) -> InitializedTitleExtractor {
        return InitializedTitleExtractor { title: title };
    }
}

impl InitializedTitleExtractor {
    // Extract music title and artist from video title
    pub fn extract_from_title(&self) -> Result<FinishedTitleExtractor, String> {
        //TODO get rid of unwraps

        //replace all | & @ \\ \" / (n spaces to 1 sapace)
        //-> any sequence of special charaters is a seperator, getting rid of all sequence of spaces with one space
        let space_regex = match Regex::new(SPACE_REGEX) {
            Ok(some) => some,
            Err(e) => return Err(format!("Error creating regex {}, {}", SPACE_REGEX, e)),
        };

        let title_seperator_regex = match Regex::new(TITLE_SEPERATOR_REGEX) {
            Ok(regex) => regex,
            Err(e) => {
                return Err(format!(
                    "Error creating regex {}, {}",
                    TITLE_SEPERATOR_REGEX, e
                ))
            }
        };

        let title: String = space_regex
            .replace_all(&self.title, " ")
            .deref()
            .to_string();

        let song_info: (String, String) = match title_seperator_regex.captures(&self.title) {
            Some(matches) => {
                match matches.iter().next() {
                    Some(match) => {
                        // can safely unwrap since first match is guaranteed to be non-null
                                            let split_match = matches.get(matches.len() - 1).unwrap();

                                            let song_name = title.chars().take(split_match.start()).collect::<String>();
                                            let song_artist = title
                                                .chars()
                                                .skip(split_match.end() + 1)
                                                .take(&self.title.len() - split_match.end() - 1)
                                                .collect::<String>();

                                            (song_name, song_artist)

                    }
                    None => {

                        return Err(format!(
                            "no matches found for title {}, ...TODO",
                            self.title
                        ));
                    }
                }
            }
            None => {
                return Err(format!(
                    "no matches found for title {}, ...TODO",
                    self.title
                ));
            }
        };

        let finished_title_extractor = FinishedTitleExtractor {
            name: song_info.0.to_owned(),
            artist: song_info.1.to_owned(),
        };

        return Ok(finished_title_extractor);
    }
}
