use super::data::urdu::URDU_UNIQUE_MARKERS;

/// Detect Urdu-specific characters.
///
/// This stays conservative and only matches characters that are commonly
/// used to distinguish Urdu from generic Arabic-script text.
pub fn is_urdu(ch: char) -> bool {
    URDU_UNIQUE_MARKERS.contains(&ch)
}
