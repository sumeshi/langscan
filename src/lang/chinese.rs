use super::markers::chinese_simplified::SIMPLIFIED_MARKERS;
use super::markers::chinese_traditional::TRADITIONAL_MARKERS;
use super::Lang;
use std::collections::BTreeMap;

pub fn is_simplified_marker(ch: char) -> bool {
    SIMPLIFIED_MARKERS.binary_search(&ch).is_ok()
}

pub fn is_traditional_marker(ch: char) -> bool {
    TRADITIONAL_MARKERS.binary_search(&ch).is_ok()
}

pub fn built_in_keywords() -> BTreeMap<Lang, Vec<String>> {
    let mut map = BTreeMap::new();
    map.insert(
        Lang::ZhHans,
        SIMPLIFIED_MARKERS.iter().map(|&c| c.to_string()).collect(),
    );
    map.insert(
        Lang::ZhHant,
        TRADITIONAL_MARKERS.iter().map(|&c| c.to_string()).collect(),
    );
    map
}
