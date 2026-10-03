use std::path::Path;
use lofty::prelude::*;
use lofty::probe::Probe;

#[derive(Debug, Clone, Default)]
pub struct TrackTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub track: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub artwork: Option<Vec<u8>>,
}

impl TrackTags {
    pub fn from_file(path: &Path) -> Self {
        let mut tags = TrackTags::default();

        let tagged = match Probe::open(path).and_then(|p| p.read()) {
            Ok(t) => t,
            Err(_) => return tags,
        };

        if let Some(tag) = tagged.primary_tag() {
            tags.title = tag.title().map(|s| s.to_string());
            tags.artist = tag.artist().map(|s| s.to_string());
            tags.album = tag.album().map(|s| s.to_string());
            tags.year = tag.year().map(|y| y.to_string());
            tags.track = tag.track().map(|t| t.to_string());
            tags.genre = tag.genre().map(|s| s.to_string());
            tags.comment = tag.comment().map(|s| s.to_string());

            if let Some(picture) = tag.pictures().first() {
                tags.artwork = Some(picture.data().to_vec());
            }
        }

        if tags.artwork.is_none() {
            for tag in tagged.tags() {
                if let Some(picture) = tag.pictures().first() {
                    tags.artwork = Some(picture.data().to_vec());
                    break;
                }
            }
        }

        tags
    }
}
