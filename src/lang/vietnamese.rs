use super::markers::vietnamese::{VIETNAMESE_MARKERS, VIETNAMESE_TONE_RANGE};

pub fn is_vietnamese(ch: char) -> bool {
    (super::is_latin_extended(ch) && VIETNAMESE_MARKERS.contains(&ch)) || {
        let (start, end) = VIETNAMESE_TONE_RANGE;
        (start..=end).contains(&ch)
    }
}
