use super::markers::japanese::JAPANESE_RANGES;

pub fn is_japanese(ch: char) -> bool {
    JAPANESE_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&ch))
}
