use super::markers::polish::POLISH_MARKERS;

pub fn is_polish(ch: char) -> bool {
    POLISH_MARKERS.contains(&ch)
}
