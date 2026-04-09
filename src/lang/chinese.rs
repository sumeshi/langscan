use super::data::chinese_simplified::SIMPLIFIED_MARKERS;
use super::data::chinese_traditional::TRADITIONAL_MARKERS;

pub fn is_simplified_marker(ch: char) -> bool {
    SIMPLIFIED_MARKERS.contains(&ch)
}

pub fn is_traditional_marker(ch: char) -> bool {
    TRADITIONAL_MARKERS.contains(&ch)
}
