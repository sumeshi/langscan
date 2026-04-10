use std::collections::BTreeMap;

use crate::lang::Lang;

use super::detect::is_highlight_char;

pub fn highlight_line(
    line: &str,
    langs: &[Lang],
    keyword_map: &BTreeMap<Lang, Vec<String>>,
) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut mark = vec![false; chars.len()];

    for (i, ch) in chars.iter().enumerate() {
        for lang in langs {
            if is_highlight_char(*lang, *ch) {
                mark[i] = true;
                break;
            }
        }
    }

    let char_offsets: Vec<usize> = line.char_indices().map(|(i, _)| i).collect();

    for lang in langs {
        if let Some(list) = keyword_map.get(lang) {
            for kw in list {
                for (start, _) in line.match_indices(kw) {
                    let end = start + kw.len();
                    let char_start = char_offsets.binary_search(&start).unwrap_or_else(|i| i);
                    let char_end = char_offsets.binary_search(&end).unwrap_or_else(|i| i);
                    for i in char_start..char_end {
                        mark[i] = true;
                    }
                }
            }
        }
    }

    let mut out = String::new();
    let mut in_mark = false;
    for (i, ch) in chars.iter().enumerate() {
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
