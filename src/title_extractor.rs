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
            Some(capture_group) => {
                // get last match in capture group
                // can safely unwrap since first match is guaranteed to be non-null
                let split_match = capture_group.get(capture_group.len() - 1).unwrap();

                let song_artist = title.chars().take(split_match.start()).collect::<String>();
                let song_name = title
                    .chars()
                    .skip(split_match.end() + 1)
                    .take(&self.title.len() - split_match.end() - 1)
                    .collect::<String>();

                (song_name, song_artist)
            }
            None => (self.title.to_owned(), "".to_string()),
        };

        let song_name = song_info.0.trim();
        let song_artist = song_info.1.trim();

        let finished_title_extractor = FinishedTitleExtractor {
            name: song_name.to_owned(),
            artist: song_artist.to_owned(),
        };

        return Ok(finished_title_extractor);
    }
}

#[cfg(test)]
mod tests {
    use super::EmptyTitleExtractor;

    #[test]
    fn test_title_extractor() {
        let empty_title_extractor =
            EmptyTitleExtractor::init("Astro - Opium Remix (Slowed)".to_string());
        let finalized_title_extractor = empty_title_extractor.extract_from_title().unwrap();

        assert_eq!(finalized_title_extractor.artist(), "Astro");
        assert_eq!(finalized_title_extractor.name(), "Opium Remix (Slowed)");

        let empty_title_extractor = EmptyTitleExtractor::init("HIMG".to_string());
        let finalized_title_extractor = empty_title_extractor.extract_from_title().unwrap();

        assert_eq!(finalized_title_extractor.artist(), "");
        assert_eq!(finalized_title_extractor.name(), "HIMG");

        let empty_title_extractor = EmptyTitleExtractor::init(
            "MOONDEITY x INTERWORLD - ONE CHANCE | SLOWED + REVERBED".to_string(),
        );
        let finalized_title_extractor = empty_title_extractor.extract_from_title().unwrap();

        assert_eq!(finalized_title_extractor.artist(), "MOONDEITY x INTERWORLD");
        assert_eq!(
            finalized_title_extractor.name(),
            "ONE CHANCE | SLOWED + REVERBED"
        );

        let empty_title_extractor = EmptyTitleExtractor::init(
            "seekae - test & recognize [ flume re - work ] slowed".to_string(),
        );
        let finalized_title_extractor = empty_title_extractor.extract_from_title().unwrap();

        assert_eq!(finalized_title_extractor.artist(), "seekae");
        assert_eq!(
            finalized_title_extractor.name(),
            "test & recognize [ flume re - work ] slowed"
        );
    }
}
