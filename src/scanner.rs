use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead};

use crate::lang::{self, Lang};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchMode {
    Matched,
    Unmatched,
}

#[derive(Default, Debug)]
struct Flags {
    has_cjk: bool,
    has_cyr: bool,
    has_hangul: bool,
    has_vietnamese: bool,
    has_thai: bool,
    has_arabic: bool,
    has_persian: bool,
    has_hebrew: bool,
    has_devanagari: bool,
    has_greek: bool,
    has_urdu: bool,
    has_japanese: bool,
    has_simplified_marker: bool,
    has_traditional_marker: bool,
}

#[derive(Default, Debug)]
struct ScanPlan {
    need_cjk: bool,
    need_cyr: bool,
    need_hangul: bool,
    need_vietnamese: bool,
    need_thai: bool,
    need_arabic: bool,
    need_persian: bool,
    need_hebrew: bool,
    need_devanagari: bool,
    need_greek: bool,
    need_urdu: bool,
    need_japanese: bool,
    need_simplified_marker: bool,
    need_traditional_marker: bool,
}

impl ScanPlan {
    fn from_langs(langs: &[Lang]) -> Self {
        let mut plan = Self::default();
        for lang in langs {
            match lang {
                Lang::Cjk => plan.need_cjk = true,
                Lang::ZhHans => {
                    plan.need_cjk = true;
                    plan.need_simplified_marker = true;
                }
                Lang::ZhHant => {
                    plan.need_cjk = true;
                    plan.need_traditional_marker = true;
                }
                Lang::Ru => plan.need_cyr = true,
                Lang::Ko | Lang::KoKp | Lang::KoKr => plan.need_hangul = true,
                Lang::Ja => {
                    plan.need_cjk = true;
                    plan.need_japanese = true;
                }
                Lang::Vi => plan.need_vietnamese = true,
                Lang::Th => plan.need_thai = true,
                Lang::Ar => plan.need_arabic = true,
                Lang::Fa => plan.need_persian = true,
                Lang::He => plan.need_hebrew = true,
                Lang::Hi => plan.need_devanagari = true,
                Lang::El => plan.need_greek = true,
                Lang::Ur => plan.need_urdu = true,
            }
        }
        plan
    }

    fn is_complete(&self, flags: &Flags) -> bool {
        (!self.need_cjk || flags.has_cjk)
            && (!self.need_cyr || flags.has_cyr)
            && (!self.need_hangul || flags.has_hangul)
            && (!self.need_japanese || flags.has_japanese)
            && (!self.need_vietnamese || flags.has_vietnamese)
            && (!self.need_thai || flags.has_thai)
            && (!self.need_arabic || flags.has_arabic)
            && (!self.need_persian || flags.has_persian)
            && (!self.need_hebrew || flags.has_hebrew)
            && (!self.need_devanagari || flags.has_devanagari)
            && (!self.need_greek || flags.has_greek)
            && (!self.need_urdu || flags.has_urdu)
            && (!self.need_simplified_marker || flags.has_simplified_marker)
            && (!self.need_traditional_marker || flags.has_traditional_marker)
    }
}

#[derive(Debug)]
pub struct LineHit {
    pub line_no: usize,
    pub labels: Vec<Lang>,
    pub highlighted: String,
}

pub fn scan_collect(
    mut reader: Box<dyn BufRead>,
    langs: &[Lang],
    keyword_map: &BTreeMap<Lang, Vec<String>>,
    mode: MatchMode,
) -> io::Result<Vec<LineHit>> {
    let plan = ScanPlan::from_langs(langs);
    let mut line_no: usize = 0;
    let mut buf: Vec<u8> = Vec::new();

    let mut hits_out: Vec<LineHit> = Vec::new();
    loop {
        buf.clear();
        let bytes = reader.read_until(b'\n', &mut buf)?;
        if bytes == 0 {
            break;
        }
        line_no += 1;

        let line_lossy = String::from_utf8_lossy(&buf);
        let line = line_lossy.trim_end_matches(['\r', '\n']);

        let flags = scan_line_flags(line, &plan);
        if should_skip_line(&flags, langs, keyword_map, line) {
            if matches!(mode, MatchMode::Unmatched) {
                hits_out.push(LineHit {
                    line_no,
                    labels: Vec::new(),
                    highlighted: line.to_string(),
                });
            }
            continue;
        }

        let mut hits: BTreeSet<Lang> = BTreeSet::new();
        for lang in langs {
            if lang_matches(*lang, line, &flags, keyword_map) {
                hits.insert(*lang);
            }
        }

        let matched = !hits.is_empty();
        if matches!(mode, MatchMode::Matched) && matched {
            let hit_vec = hits.into_iter().collect::<Vec<_>>();
            let highlighted = highlight_line(line, &hit_vec, keyword_map);
            hits_out.push(LineHit {
                line_no,
                labels: hit_vec,
                highlighted,
            });
        } else if matches!(mode, MatchMode::Unmatched) && !matched {
            hits_out.push(LineHit {
                line_no,
                labels: Vec::new(),
                highlighted: line.to_string(),
            });
        }
    }

    Ok(hits_out)
}

/// Scan a line to set per-script flags.
///
/// Only checks scripts that are relevant to the requested languages,
/// and exits early once all needed flags are satisfied.
fn scan_line_flags(line: &str, plan: &ScanPlan) -> Flags {
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

        // Early exit when all needed flags are set
        if plan.is_complete(&flags) {
            break;
        }
    }

    flags
}

/// Determine whether a line can be skipped entirely.
///
/// A line is skipped when **no** target language's Unicode flag is set
/// **and** no keyword from the keyword map matches the line text.
/// This handles the edge case where a user adds an ASCII keyword via
/// `--keyword`, which would not trigger any Unicode flag.
fn should_skip_line(
    flags: &Flags,
    langs: &[Lang],
    keyword_map: &BTreeMap<Lang, Vec<String>>,
    line: &str,
) -> bool {
    // Fast path: don't skip if any Unicode flag matches a target language
    let has_script = langs.iter().any(|lang| match lang {
        Lang::Cjk => flags.has_cjk,
        Lang::ZhHans => flags.has_cjk && flags.has_simplified_marker,
        Lang::ZhHant => flags.has_cjk && flags.has_traditional_marker,
        Lang::Ru => flags.has_cyr,
        Lang::Ko | Lang::KoKp | Lang::KoKr => flags.has_hangul,
        Lang::Ja => flags.has_japanese,
        Lang::Vi => flags.has_vietnamese,
        Lang::Th => flags.has_thai,
        Lang::Ar => flags.has_arabic,
        Lang::Fa => flags.has_persian,
        Lang::He => flags.has_hebrew,
        Lang::Hi => flags.has_devanagari,
        Lang::El => flags.has_greek,
        Lang::Ur => flags.has_urdu,
    });
    if has_script {
        return false;
    }

    // Slow path: check if any keyword matches the line text
    for lang in langs {
        if let Some(keywords) = keyword_map.get(lang) {
            if keywords.iter().any(|kw| line.contains(kw)) {
                return false;
            }
        }
    }

    true
}

fn lang_matches(
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
        Lang::Ru => flags.has_cyr,
        Lang::Ko => flags.has_hangul,
        Lang::KoKp | Lang::KoKr => flags.has_hangul,
        Lang::Ja => flags.has_japanese,
        Lang::Vi => flags.has_vietnamese,
        Lang::Th => flags.has_thai,
        Lang::Ar => flags.has_arabic,
        Lang::Fa => flags.has_persian,
        Lang::He => flags.has_hebrew,
        Lang::Hi => flags.has_devanagari,
        Lang::El => flags.has_greek,
        Lang::Ur => flags.has_urdu,
    }
}

fn highlight_line(line: &str, langs: &[Lang], keyword_map: &BTreeMap<Lang, Vec<String>>) -> String {
    let mut chars: Vec<(usize, usize, char)> = Vec::new();
    for (idx, ch) in line.char_indices() {
        let end = idx + ch.len_utf8();
        chars.push((idx, end, ch));
    }
    let mut mark = vec![false; chars.len()];

    for (i, &(_, _, ch)) in chars.iter().enumerate() {
        for lang in langs {
            if is_highlight_char(*lang, ch) {
                mark[i] = true;
                break;
            }
        }
    }

    for lang in langs {
        if let Some(list) = keyword_map.get(lang) {
            for kw in list {
                for (start, _) in line.match_indices(kw) {
                    let end = start + kw.len();
                    for (i, (c_start, c_end, _)) in chars.iter().enumerate() {
                        if *c_end <= start || *c_start >= end {
                            continue;
                        }
                        mark[i] = true;
                    }
                }
            }
        }
    }

    let mut out = String::new();
    let mut in_mark = false;
    for (i, (_, _, ch)) in chars.iter().enumerate() {
        if mark[i] && !in_mark {
            out.push_str("\x1b[31m");
            in_mark = true;
        } else if !mark[i] && in_mark {
            out.push_str("\x1b[0m");
            in_mark = false;
        }
        out.push(*ch);
    }
    if in_mark {
        out.push_str("\x1b[0m");
    }
    out
}

fn is_highlight_char(lang: Lang, ch: char) -> bool {
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
        Lang::Ar => lang::is_arabic(ch),
        Lang::Fa => lang::is_persian(ch),
        Lang::He => lang::is_hebrew(ch),
        Lang::Hi => lang::is_devanagari(ch),
        Lang::El => lang::is_greek(ch),
        Lang::Ur => lang::is_urdu(ch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn scan(text: &str, langs: &[Lang]) -> Vec<LineHit> {
        let reader: Box<dyn BufRead> = Box::new(Cursor::new(text.to_string()));
        let kw = lang::load_keywords(&[]).unwrap();
        scan_collect(reader, langs, &kw, MatchMode::Matched).unwrap()
    }

    fn scan_with_keywords(text: &str, langs: &[Lang], kw_args: &[&str]) -> Vec<LineHit> {
        let reader: Box<dyn BufRead> = Box::new(Cursor::new(text.to_string()));
        let kw_strings: Vec<String> = kw_args.iter().map(|s| s.to_string()).collect();
        let kw = lang::load_keywords(&kw_strings).unwrap();
        scan_collect(reader, langs, &kw, MatchMode::Matched).unwrap()
    }

    fn scan_unmatched(text: &str, langs: &[Lang]) -> Vec<LineHit> {
        let reader: Box<dyn BufRead> = Box::new(Cursor::new(text.to_string()));
        let kw = lang::load_keywords(&[]).unwrap();
        scan_collect(reader, langs, &kw, MatchMode::Unmatched).unwrap()
    }

    // --- Single language detection ---

    #[test]
    fn detect_japanese() {
        let hits = scan("これはテストです\n", &[Lang::Ja]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ja));
    }

    #[test]
    fn detect_cjk() {
        let hits = scan("漢字テスト\n", &[Lang::Cjk]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Cjk));
    }

    #[test]
    fn detect_simplified_chinese() {
        let hits = scan("这是测试\n", &[Lang::Cjk, Lang::ZhHans]);
        assert!(hits[0].labels.contains(&Lang::ZhHans));
    }

    #[test]
    fn detect_traditional_chinese() {
        let hits = scan("這是測試\n", &[Lang::Cjk, Lang::ZhHant]);
        assert!(hits[0].labels.contains(&Lang::ZhHant));
    }

    #[test]
    fn detect_expanded_simplified_chinese_marker() {
        let hits = scan("网络安全\n", &[Lang::Cjk, Lang::ZhHans]);
        assert!(hits[0].labels.contains(&Lang::ZhHans));
    }

    #[test]
    fn detect_expanded_traditional_chinese_marker() {
        let hits = scan("網路安全\n", &[Lang::Cjk, Lang::ZhHant]);
        assert!(hits[0].labels.contains(&Lang::ZhHant));
    }

    #[test]
    fn detect_simplified_without_explicit_cjk_target() {
        let hits = scan("ABC东\n", &[Lang::ZhHans]);
        assert!(hits[0].labels.contains(&Lang::ZhHans));
    }

    #[test]
    fn detect_korean() {
        let hits = scan("테스트입니다\n", &[Lang::Ko]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ko));
    }

    #[test]
    fn detect_russian() {
        let hits = scan("пример строки\n", &[Lang::Ru]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ru));
    }

    #[test]
    fn detect_vietnamese() {
        let hits = scan("tiếng Việt có dấu\n", &[Lang::Vi]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Vi));
    }

    #[test]
    fn detect_thai() {
        let hits = scan("ภาษาไทยทดสอบ\n", &[Lang::Th]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Th));
    }

    #[test]
    fn detect_arabic() {
        let hits = scan("العربية اختبار\n", &[Lang::Ar]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ar));
    }

    #[test]
    fn detect_persian() {
        let hits = scan("گزارش امنیتی\n", &[Lang::Fa]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Fa));
    }

    #[test]
    fn detect_hebrew() {
        let hits = scan("בדיקת אבטחה\n", &[Lang::He]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::He));
    }

    #[test]
    fn detect_urdu() {
        let hits = scan("یہ اردو ٹیسٹ ہے\n", &[Lang::Ur]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ur));
    }

    #[test]
    fn detect_hindi() {
        let hits = scan("यह एक परीक्षण है\n", &[Lang::Hi]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Hi));
    }

    #[test]
    fn detect_greek() {
        let hits = scan("Ελληνικά τεστ\n", &[Lang::El]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::El));
    }

    // --- No false positives ---

    #[test]
    fn ascii_only_no_hits() {
        let hits = scan(
            "this is a normal ascii line\n",
            &[Lang::Ja, Lang::Ru, Lang::Ko],
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn vietnamese_no_false_positive_french() {
        // French text should NOT trigger Vietnamese detection
        let hits = scan("café résumé naïve\n", &[Lang::Vi]);
        assert!(hits.is_empty());
    }

    #[test]
    fn persian_no_false_positive_arabic() {
        let hits = scan("العربية اختبار\n", &[Lang::Fa]);
        assert!(hits.is_empty());
    }

    #[test]
    fn urdu_no_false_positive_arabic() {
        let hits = scan("العربية اختبار\n", &[Lang::Ur]);
        assert!(hits.is_empty());
    }

    // --- Mixed languages ---

    #[test]
    fn detect_mixed_ja_ru() {
        let hits = scan("テスト пример\n", &[Lang::Ja, Lang::Ru]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ja));
        assert!(hits[0].labels.contains(&Lang::Ru));
    }

    #[test]
    fn detect_mixed_ko_el() {
        let hits = scan("Δοκιμή 테스트\n", &[Lang::Ko, Lang::El]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ko));
        assert!(hits[0].labels.contains(&Lang::El));
    }

    // --- DPRK/ROK keyword detection ---

    #[test]
    fn detect_dprk_keywords() {
        let hits = scan("조선 로동 동무\n", &[Lang::Ko, Lang::KoKp, Lang::KoKr]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ko));
        assert!(hits[0].labels.contains(&Lang::KoKp));
    }

    #[test]
    fn detect_rok_keywords() {
        let hits = scan("대한민국 서울\n", &[Lang::Ko, Lang::KoKp, Lang::KoKr]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ko));
        assert!(hits[0].labels.contains(&Lang::KoKr));
    }

    #[test]
    fn detect_expanded_dprk_keywords() {
        let hits = scan("평양 주체 사상\n", &[Lang::Ko, Lang::KoKp, Lang::KoKr]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::KoKp));
    }

    #[test]
    fn detect_expanded_rok_keywords() {
        let hits = scan("대통령실 브리핑\n", &[Lang::Ko, Lang::KoKp, Lang::KoKr]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::KoKr));
    }

    // --- Custom keyword via --keyword ---

    #[test]
    fn custom_ascii_keyword_not_skipped() {
        // An ASCII keyword should be detected even though no Unicode flags fire
        let hits = scan_with_keywords("malware detected here\n", &[Lang::Ru], &["ru=malware"]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ru));
    }

    // --- Multi-line ---

    #[test]
    fn multi_line_correct_line_numbers() {
        let text = "line one\nテスト\nline three\nтест\n";
        let hits = scan(text, &[Lang::Ja, Lang::Ru]);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line_no, 2);
        assert_eq!(hits[1].line_no, 4);
    }

    // --- Highlighting ---

    #[test]
    fn highlight_contains_ansi_codes() {
        let hits = scan("abc テスト def\n", &[Lang::Ja]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].highlighted.contains("\x1b[31m"));
        assert!(hits[0].highlighted.contains("\x1b[0m"));
    }

    #[test]
    fn invert_match_returns_only_unmatched_lines() {
        let hits = scan_unmatched("plain ascii\nテスト\nsecond ascii\n", &[Lang::Ja]);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line_no, 1);
        assert_eq!(hits[0].highlighted, "plain ascii");
        assert_eq!(hits[1].line_no, 3);
        assert_eq!(hits[1].highlighted, "second ascii");
        assert!(hits[0].labels.is_empty());
    }
}
