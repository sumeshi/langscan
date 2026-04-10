use std::collections::BTreeMap;

use super::markers::korean_kp::KO_KP_KEYWORDS;
use super::markers::korean_kr::KO_KR_KEYWORDS;
use super::Lang;

pub fn is_hangul(ch: char) -> bool {
    matches!(
        ch,
        '\u{1100}'..='\u{11FF}' | // Jamo
        '\u{3130}'..='\u{318F}' | // Compatibility
        '\u{AC00}'..='\u{D7A3}' | // Syllables
        '\u{A960}'..='\u{A97F}' | // Extended-A
        '\u{D7B0}'..='\u{D7FF}'   // Extended-B
    )
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
