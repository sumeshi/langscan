use super::markers::arabic::ARABIC_RANGES;

pub fn is_arabic(ch: char) -> bool {
    ARABIC_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&ch))
}
