use super::markers::greek::GREEK_RANGES;

pub fn is_greek(ch: char) -> bool {
    GREEK_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&ch))
}
