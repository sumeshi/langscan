use super::markers::hebrew::HEBREW_RANGES;

pub fn is_hebrew(ch: char) -> bool {
    HEBREW_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&ch))
}
