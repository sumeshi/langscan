use super::data::vietnamese::VIETNAMESE_UNIQUE_MARKERS;

/// Detect Vietnamese-specific characters.
///
/// Uses Vietnamese-unique marker characters from data plus the
/// Latin Extended Additional block U+1EA0..U+1EF9 to avoid false
/// positives from French, Spanish, or Portuguese text.
pub fn is_vietnamese(ch: char) -> bool {
    VIETNAMESE_UNIQUE_MARKERS.contains(&ch) || matches!(ch as u32, 0x1EA0..=0x1EF9)
}
