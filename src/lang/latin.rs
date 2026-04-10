use super::markers::latin::{LATIN_EXTENDED_A_RANGE, LATIN_EXTENDED_RANGE};

pub fn is_latin_extended(ch: char) -> bool {
    let (start, end) = LATIN_EXTENDED_RANGE;
    (start..=end).contains(&ch) || {
        let (start, end) = LATIN_EXTENDED_A_RANGE;
        (start..=end).contains(&ch)
    }
}
