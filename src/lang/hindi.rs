use super::markers::hindi::{DEVANAGARI_EXTENDED_RANGE, DEVANAGARI_RANGE};

pub fn is_devanagari(ch: char) -> bool {
    let (start, end) = DEVANAGARI_RANGE;
    (start..=end).contains(&ch) || {
        let (start, end) = DEVANAGARI_EXTENDED_RANGE;
        (start..=end).contains(&ch)
    }
}
