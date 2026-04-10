use std::collections::BTreeMap;
use std::io;

use crate::lang;
use crate::scanner::LineHit;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    JsonLines,
    Yaml,
}

#[derive(serde::Serialize)]
pub struct JsonFileResult {
    pub file: Option<String>,
    pub hits: BTreeMap<String, Vec<usize>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub misses: Vec<usize>,
}

#[derive(serde::Serialize)]
pub struct JsonLineResult {
    pub file: Option<String>,
    pub line: usize,
    pub content: String,
    pub labels: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct YamlFileResult {
    pub file: Option<String>,
    pub hits: BTreeMap<String, Vec<usize>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub misses: Vec<usize>,
}

pub fn print_serialized_results(
    format: OutputFormat,
    file: Option<&str>,
    lines: &[LineHit],
) -> io::Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&vec![JsonFileResult {
                file: file.map(str::to_string),
                hits: summarize_hits(lines),
                misses: summarize_misses(lines),
            }])
            .map_err(io::Error::other)?;
            println!("{json}");
        }
        OutputFormat::JsonLines => {
            for hit in lines {
                let json = serde_json::to_string(&JsonLineResult {
                    file: file.map(str::to_string),
                    line: hit.line_no,
                    content: hit.original.clone(),
                    labels: hit
                        .labels
                        .iter()
                        .copied()
                        .map(|l| lang::lang_label(l).to_string())
                        .collect(),
                })
                .map_err(io::Error::other)?;
                println!("{json}");
            }
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&vec![YamlFileResult {
                file: file.map(str::to_string),
                hits: summarize_hits(lines),
                misses: summarize_misses(lines),
            }])
            .map_err(io::Error::other)?;
            print!("{yaml}");
        }
        OutputFormat::Text => {
            output_results(format, file, lines, file.is_some(), false)?;
        }
    }

    Ok(())
}

pub fn collect_serialized_file_results(
    format: OutputFormat,
    file: &str,
    lines: &[LineHit],
    json_out: &mut Vec<JsonFileResult>,
    yaml_out: &mut Vec<YamlFileResult>,
) {
    match format {
        OutputFormat::Json => json_out.push(JsonFileResult {
            file: Some(file.to_string()),
            hits: summarize_hits(lines),
            misses: summarize_misses(lines),
        }),
        OutputFormat::Yaml => yaml_out.push(YamlFileResult {
            file: Some(file.to_string()),
            hits: summarize_hits(lines),
            misses: summarize_misses(lines),
        }),
        OutputFormat::Text | OutputFormat::JsonLines => {}
    }
}

pub fn flush_serialized_file_results(
    format: OutputFormat,
    json_out: &[JsonFileResult],
    yaml_out: &[YamlFileResult],
) -> io::Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(json_out).map_err(io::Error::other)?;
            println!("{json}");
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(yaml_out).map_err(io::Error::other)?;
            print!("{yaml}");
        }
        OutputFormat::Text | OutputFormat::JsonLines => {}
    }

    Ok(())
}

pub fn output_results(
    format: OutputFormat,
    file: Option<&str>,
    lines: &[LineHit],
    show_file: bool,
    invert_match: bool,
) -> io::Result<()> {
    match format {
        OutputFormat::Text => {
            for hit in lines {
                if invert_match {
                    if show_file {
                        let label = file.unwrap_or("-");
                        println!("[{}:L{}] {}", label, hit.line_no, hit.highlighted);
                    } else {
                        println!("[L{}] {}", hit.line_no, hit.highlighted);
                    }
                } else {
                    let lang_list = hit
                        .labels
                        .iter()
                        .copied()
                        .map(lang::lang_label)
                        .collect::<Vec<_>>()
                        .join(",");
                    if show_file {
                        let label = file.unwrap_or("-");
                        println!(
                            "[{}:L{}:{}] {}",
                            label, hit.line_no, lang_list, hit.highlighted
                        );
                    } else {
                        println!("[L{}:{}] {}", hit.line_no, lang_list, hit.highlighted);
                    }
                }
            }
        }
        OutputFormat::Json => {}
        OutputFormat::JsonLines => {}
        OutputFormat::Yaml => {}
    }
    Ok(())
}

pub fn summarize_hits(lines: &[LineHit]) -> BTreeMap<String, Vec<usize>> {
    let mut out: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for hit in lines {
        for lang in &hit.labels {
            let label = lang::lang_label(*lang).to_string();
            out.entry(label).or_default().push(hit.line_no);
        }
    }
    out
}

pub fn summarize_misses(lines: &[LineHit]) -> Vec<usize> {
    lines
        .iter()
        .filter(|hit| hit.labels.is_empty())
        .map(|hit| hit.line_no)
        .collect()
}

pub fn count_by_lang(lines: &[LineHit]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for hit in lines {
        for lang in &hit.labels {
            let label = lang::lang_label(*lang).to_string();
            *counts.entry(label).or_default() += 1;
        }
    }
    counts.into_iter().collect()
}

pub fn print_counts(lines: &[LineHit]) {
    let mut counts = count_by_lang(lines);
    if counts.is_empty() {
        return;
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    let total: usize = counts.iter().map(|(_, c)| c).sum();
    let max_label_len = counts.iter().map(|(s, _)| s.len()).max().unwrap_or(0);
    let col_width = max_label_len.max(8);
    println!("+-{}-+------+", "-".repeat(col_width));
    println!("| {:col_width$} | {:>4} |", "LANG", "COUNT");
    println!("+-{}-+------+", "-".repeat(col_width));
    for (lang, count) in &counts {
        println!("| {:col_width$} | {:>4} |", lang, count);
    }
    println!("+-{}-+------+", "-".repeat(col_width));
    println!("| {:col_width$} | {:>4} |", "TOTAL", total);
    println!("+-{}-+------+", "-".repeat(col_width));
}

pub fn print_counts_for_file(file: &str, lines: &[LineHit]) {
    let mut counts = count_by_lang(lines);
    if counts.is_empty() {
        return;
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    let total: usize = counts.iter().map(|(_, c)| c).sum();
    let max_label_len = counts.iter().map(|(s, _)| s.len()).max().unwrap_or(0);
    let col_width = max_label_len.max(8);
    println!("{}:", file);
    println!("+-{}-+------+", "-".repeat(col_width));
    println!("| {:col_width$} | {:>4} |", "LANG", "COUNT");
    println!("+-{}-+------+", "-".repeat(col_width));
    for (lang, count) in &counts {
        println!("| {:col_width$} | {:>4} |", lang, count);
    }
    println!("+-{}-+------+", "-".repeat(col_width));
    println!("| {:col_width$} | {:>4} |", "TOTAL", total);
    println!("+-{}-+------+", "-".repeat(col_width));
}

pub fn print_total_counts(lines: &[LineHit]) {
    println!("\n=== TOTAL ===\n");
    print_counts(lines);
}
