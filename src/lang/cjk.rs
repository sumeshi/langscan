use super::markers::cjk::CJK_RANGES;

pub fn is_cjk(ch: char) -> bool {
    CJK_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&ch))
}
