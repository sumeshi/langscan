use std::collections::BTreeMap;

use crate::lang::{self, Lang};

use super::flags::Flags;
use super::highlight::highlight_line;

pub fn lang_matches(
    lang: Lang,
    line: &str,
    flags: &Flags,
    keyword_map: &BTreeMap<Lang, Vec<String>>,
) -> bool {
    let keywords = keyword_map.get(&lang);
    if let Some(list) = keywords {
        if list.iter().any(|kw| line.contains(kw)) {
            return true;
        }
        if matches!(lang, Lang::KoKp | Lang::KoKr) {
            return false;
        }
    }

    match lang {
        Lang::Cjk => flags.has_cjk,
        Lang::ZhHans => flags.has_simplified_marker,
        Lang::ZhHant => flags.has_traditional_marker,
        Lang::Ru => flags.has_cyr && !flags.has_ukrainian,
        Lang::Ko => flags.has_hangul,
        Lang::KoKp | Lang::KoKr => flags.has_hangul,
        Lang::Ja => flags.has_japanese,
        Lang::Vi => flags.has_vietnamese,
        Lang::Th => flags.has_thai,
        Lang::Tr => flags.has_turkish,
        Lang::Uk => flags.has_ukrainian,
        Lang::Ar => flags.has_arabic,
        Lang::Fa => flags.has_persian,
        Lang::He => flags.has_hebrew,
        Lang::Hi => flags.has_devanagari,
        Lang::El => flags.has_greek,
        Lang::Pl => flags.has_polish && !flags.has_vietnamese,
        Lang::Ur => flags.has_urdu,
    }
}

pub fn is_highlight_char(lang: Lang, ch: char) -> bool {
    match lang {
        Lang::Cjk => lang::is_cjk(ch),
        Lang::ZhHans => lang::is_simplified_marker(ch),
        Lang::ZhHant => lang::is_traditional_marker(ch),
        Lang::Ru => lang::is_cyrillic(ch),
        Lang::Ko => lang::is_hangul(ch),
        Lang::KoKp | Lang::KoKr => false,
        Lang::Ja => lang::is_japanese(ch),
        Lang::Vi => lang::is_vietnamese(ch),
        Lang::Th => lang::is_thai(ch),
        Lang::Tr => lang::is_turkish(ch),
        Lang::Uk => lang::is_ukrainian(ch),
        Lang::Ar => lang::is_arabic(ch),
        Lang::Fa => lang::is_persian(ch),
        Lang::He => lang::is_hebrew(ch),
        Lang::Hi => lang::is_devanagari(ch),
        Lang::El => lang::is_greek(ch),
        Lang::Pl => lang::is_polish(ch),
        Lang::Ur => lang::is_urdu(ch),
    }
}

pub fn should_skip_line(
    flags: &Flags,
    langs: &[Lang],
    keyword_map: &BTreeMap<Lang, Vec<String>>,
    line: &str,
) -> bool {
    let has_script = langs.iter().any(|lang| match lang {
        Lang::Cjk => flags.has_cjk,
        Lang::ZhHans => flags.has_cjk && flags.has_simplified_marker,
        Lang::ZhHant => flags.has_cjk && flags.has_traditional_marker,
        Lang::Ru => flags.has_cyr,
        Lang::Ko | Lang::KoKp | Lang::KoKr => flags.has_hangul,
        Lang::Ja => flags.has_japanese,
        Lang::Vi => flags.has_vietnamese,
        Lang::Th => flags.has_thai,
        Lang::Tr => flags.has_turkish,
        Lang::Uk => flags.has_ukrainian,
        Lang::Ar => flags.has_arabic,
        Lang::Fa => flags.has_persian,
        Lang::He => flags.has_hebrew,
        Lang::Hi => flags.has_devanagari,
        Lang::El => flags.has_greek,
        Lang::Pl => flags.has_polish,
        Lang::Ur => flags.has_urdu,
    });
    if has_script {
        return false;
    }

    for lang in langs {
        if let Some(keywords) = keyword_map.get(lang) {
            if keywords.iter().any(|kw| line.contains(kw)) {
                return false;
            }
        }
    }

    true
}

pub fn scan_line_flags(line: &str, plan: &super::flags::ScanPlan) -> Flags {
    use super::flags::Flags;

    let mut flags = Flags::default();

    for ch in line.chars() {
        if plan.need_cjk && !flags.has_cjk && lang::is_cjk(ch) {
            flags.has_cjk = true;
        }
        if plan.need_cyr && !flags.has_cyr && lang::is_cyrillic(ch) {
            flags.has_cyr = true;
        }
        if plan.need_hangul && !flags.has_hangul && lang::is_hangul(ch) {
            flags.has_hangul = true;
        }
        if plan.need_japanese && !flags.has_japanese && lang::is_japanese(ch) {
            flags.has_japanese = true;
        }
        if plan.need_vietnamese && !flags.has_vietnamese && lang::is_vietnamese(ch) {
            flags.has_vietnamese = true;
        }
        if plan.need_thai && !flags.has_thai && lang::is_thai(ch) {
            flags.has_thai = true;
        }
        if plan.need_turkish && !flags.has_turkish && lang::is_turkish(ch) {
            flags.has_turkish = true;
        }
        if plan.need_ukrainian && !flags.has_ukrainian && lang::is_ukrainian(ch) {
            flags.has_ukrainian = true;
        }
        if plan.need_arabic && !flags.has_arabic && lang::is_arabic(ch) {
            flags.has_arabic = true;
        }
        if plan.need_persian && !flags.has_persian && lang::is_persian(ch) {
            flags.has_persian = true;
        }
        if plan.need_hebrew && !flags.has_hebrew && lang::is_hebrew(ch) {
            flags.has_hebrew = true;
        }
        if plan.need_devanagari && !flags.has_devanagari && lang::is_devanagari(ch) {
            flags.has_devanagari = true;
        }
        if plan.need_greek && !flags.has_greek && lang::is_greek(ch) {
            flags.has_greek = true;
        }
        if plan.need_polish && !flags.has_polish && lang::is_polish(ch) {
            flags.has_polish = true;
        }
        if plan.need_urdu && !flags.has_urdu && lang::is_urdu(ch) {
            flags.has_urdu = true;
        }
        if plan.need_simplified_marker
            && !flags.has_simplified_marker
            && lang::is_simplified_marker(ch)
        {
            flags.has_simplified_marker = true;
        }
        if plan.need_traditional_marker
            && !flags.has_traditional_marker
            && lang::is_traditional_marker(ch)
        {
            flags.has_traditional_marker = true;
        }

        if plan.is_complete(&flags) {
            break;
        }
    }

    flags
}

pub fn run_detect(
    line: &str,
    langs: &[Lang],
    flags: &Flags,
    keyword_map: &BTreeMap<Lang, Vec<String>>,
) -> Vec<Lang> {
    let mut hits: Vec<Lang> = Vec::new();
    for lang in langs {
        if lang_matches(*lang, line, flags, keyword_map) {
            hits.push(*lang);
        }
    }
    hits.sort_by_key(|lang| lang::lang_label(*lang));
    hits
}

pub fn create_line_hit(
    line_no: usize,
    line: &str,
    hits: Vec<Lang>,
    keyword_map: &BTreeMap<Lang, Vec<String>>,
) -> super::LineHit {
    let original = line.to_string();
    let highlighted = highlight_line(line, &hits, keyword_map);
    super::LineHit {
        line_no,
        labels: hits,
        original,
        highlighted,
    }
}
