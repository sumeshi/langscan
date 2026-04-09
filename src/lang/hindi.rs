pub fn is_devanagari(ch: char) -> bool {
    matches!(ch as u32, 0x0900..=0x097F)
}
