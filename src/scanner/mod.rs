use std::collections::BTreeMap;
use std::io::{self, BufRead};

use crate::lang::Lang;

pub mod detect;
pub mod flags;
pub mod highlight;

pub use detect::should_skip_line;
pub use flags::{MatchMode, ScanPlan};

#[derive(Debug, Clone)]
pub struct LineHit {
    pub line_no: usize,
    pub labels: Vec<Lang>,
    pub original: String,
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

        let flags = detect::scan_line_flags(line, &plan);
        if should_skip_line(&flags, langs, keyword_map, line) {
            if matches!(mode, MatchMode::Unmatched) {
                hits_out.push(LineHit {
                    line_no,
                    labels: Vec::new(),
                    original: line.to_string(),
                    highlighted: line.to_string(),
                });
            }
            continue;
        }

        let hits = detect::run_detect(line, langs, &flags, keyword_map);

        let matched = !hits.is_empty();
        if matches!(mode, MatchMode::Matched) && matched {
            hits_out.push(detect::create_line_hit(line_no, line, hits, keyword_map));
        } else if matches!(mode, MatchMode::Unmatched) && !matched {
            hits_out.push(LineHit {
                line_no,
                labels: Vec::new(),
                original: line.to_string(),
                highlighted: line.to_string(),
            });
        }
    }

    Ok(hits_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang;
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

    #[test]
    fn detect_japanese() {
        let hits = scan("これはテストです\n", &[Lang::Ja]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ja));
    }

    #[test]
    fn detect_single_hiragana_japanese() {
        let hits = scan("あ\n", &[Lang::Ja]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ja));
    }

    #[test]
    fn detect_single_cyrillic_yo() {
        let hits = scan("ё\n", &[Lang::Ru]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ru));
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
        let hits = scan("權限安全\n", &[Lang::Cjk, Lang::ZhHant]);
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

    #[test]
    fn detect_turkish() {
        let hits = scan("Şifre çözüldü\n", &[Lang::Tr]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Tr));
    }

    #[test]
    fn detect_ukrainian() {
        let hits = scan("Перемога України\n", &[Lang::Uk]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Uk));
    }

    #[test]
    fn ukrainian_does_not_overlap_russian() {
        let hits = scan("Україна\n", &[Lang::Ru, Lang::Uk]);
        assert_eq!(hits.len(), 1);
        assert!(!hits[0].labels.contains(&Lang::Ru));
        assert!(hits[0].labels.contains(&Lang::Uk));
    }

    #[test]
    fn belarusian_short_u_is_not_ukrainian() {
        let hits = scan("Ў\n", &[Lang::Uk]);
        assert!(hits.is_empty());
    }

    #[test]
    fn detect_polish() {
        let hits = scan("Dzień dobry\n", &[Lang::Pl]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Pl));
    }

    #[test]
    fn polish_o_acute_still_matches_polish() {
        let hits = scan("ó\n", &[Lang::Pl]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Pl));
    }

    #[test]
    fn polish_does_not_overlap_vietnamese() {
        let hits = scan("tiếng Việt có dấu\n", &[Lang::Pl, Lang::Vi]);
        assert_eq!(hits.len(), 1);
        assert!(!hits[0].labels.contains(&Lang::Pl));
        assert!(hits[0].labels.contains(&Lang::Vi));
    }

    #[test]
    fn detect_single_char_representatives() {
        let cases = [
            ("א\n", Lang::He),
            ("ا\n", Lang::Ar),
            ("अ\n", Lang::Hi),
            ("ก\n", Lang::Th),
            ("α\n", Lang::El),
            ("ğ\n", Lang::Tr),
            ("ї\n", Lang::Uk),
            ("ں\n", Lang::Ur),
            ("پ\n", Lang::Fa),
            ("ą\n", Lang::Pl),
            ("ơ\n", Lang::Vi),
            ("한\n", Lang::Ko),
            ("东\n", Lang::Cjk),
        ];

        for (text, lang) in cases {
            let hits = scan(text, &[lang]);
            assert_eq!(hits.len(), 1, "expected single-char detection for {lang:?}");
            assert!(
                hits[0].labels.contains(&lang),
                "expected label {lang:?} for {:?}",
                text.trim_end()
            );
        }
    }

    #[test]
    fn detect_arabic_presentation_form_b() {
        let hits = scan("ﻙ\n", &[Lang::Ar]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ar));
    }

    #[test]
    fn turkish_no_false_positive_vietnamese() {
        let hits = scan("tiếng Việt có dấu\n", &[Lang::Tr]);
        assert!(hits.is_empty());
    }

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

    #[test]
    fn custom_ascii_keyword_not_skipped() {
        let hits = scan_with_keywords("malware detected here\n", &[Lang::Ru], &["ru=malware"]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].labels.contains(&Lang::Ru));
    }

    #[test]
    fn multi_line_correct_line_numbers() {
        let text = "line one\nテスト\nline three\nтест\n";
        let hits = scan(text, &[Lang::Ja, Lang::Ru]);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line_no, 2);
        assert_eq!(hits[1].line_no, 4);
    }

    #[test]
    fn highlight_contains_ansi_codes() {
        let hits = scan("abc テスト def\n", &[Lang::Ja]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].highlighted.contains("\x1b[31m"));
        assert!(hits[0].highlighted.contains("\x1b[0m"));
    }

    #[test]
    fn labels_are_sorted_alphabetically() {
        let hits = scan(
            "安全测试 테스트 العربية\n",
            &[Lang::Ko, Lang::Ar, Lang::Cjk, Lang::ZhHans],
        );
        let labels = hits[0]
            .labels
            .iter()
            .copied()
            .map(lang::lang_label)
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["ar", "cjk", "cn", "ko"]);
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
