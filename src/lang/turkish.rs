use super::markers::turkish::TURKISH_MARKERS;

pub fn is_turkish(ch: char) -> bool {
    TURKISH_MARKERS.contains(&ch)
}
