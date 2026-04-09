pub fn is_greek(ch: char) -> bool {
    matches!(ch as u32, 0x0370..=0x03FF | 0x1F00..=0x1FFF)
}
