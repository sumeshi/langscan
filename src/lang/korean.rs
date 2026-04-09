use std::collections::BTreeMap;

use super::data::korean_kp::KO_KP_KEYWORDS;
use super::data::korean_kr::KO_KR_KEYWORDS;
use super::Lang;

pub fn is_hangul(ch: char) -> bool {
    matches!(ch as u32, 0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7A3)
}

pub fn built_in_keywords() -> BTreeMap<Lang, Vec<String>> {
    let mut map: BTreeMap<Lang, Vec<String>> = BTreeMap::new();
    map.insert(
        Lang::KoKp,
        KO_KP_KEYWORDS.iter().map(|s| s.to_string()).collect(),
    );
    map.insert(
        Lang::KoKr,
        KO_KR_KEYWORDS.iter().map(|s| s.to_string()).collect(),
    );
    map
}
