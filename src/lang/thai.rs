pub fn is_thai(ch: char) -> bool {
    matches!(ch as u32, 0x0E00..=0x0E7F)
}
