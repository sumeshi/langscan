use super::markers::thai::THAI_RANGE;

pub fn is_thai(ch: char) -> bool {
    let (start, end) = THAI_RANGE;
    (start..=end).contains(&ch)
}
