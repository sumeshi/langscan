use super::data::persian::PERSIAN_UNIQUE_MARKERS;

/// Detect Persian-specific characters.
///
/// This is intentionally conservative and only matches characters that are
/// strongly associated with Persian usage to avoid collapsing into generic
/// Arabic-script detection.
pub fn is_persian(ch: char) -> bool {
    PERSIAN_UNIQUE_MARKERS.contains(&ch)
}
