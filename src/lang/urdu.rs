use super::markers::urdu::URDU_MARKERS;

pub fn is_urdu(ch: char) -> bool {
    URDU_MARKERS.contains(&ch)
}
