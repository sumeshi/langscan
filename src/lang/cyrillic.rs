use super::markers::cyrillic::CYRILLIC_RANGES;

pub fn is_cyrillic(ch: char) -> bool {
    CYRILLIC_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&ch))
}
