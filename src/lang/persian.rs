use super::markers::persian::PERSIAN_MARKERS;

pub fn is_persian(ch: char) -> bool {
    PERSIAN_MARKERS.contains(&ch)
}
