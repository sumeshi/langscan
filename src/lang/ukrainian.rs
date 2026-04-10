use super::markers::ukrainian::UKRAINIAN_MARKERS;

pub fn is_ukrainian(ch: char) -> bool {
    UKRAINIAN_MARKERS.contains(&ch)
}
