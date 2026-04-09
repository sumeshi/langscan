use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use walkdir::WalkDir;

mod lang;
mod scanner;

#[derive(Parser, Debug)]
#[command(name = "langscan")]
#[command(about = "Scan lines for uncommon scripts (CJK, Cyrillic) and language hints")]
struct Args {
    /// Input file or directory (defaults to stdin)
    input: Vec<PathBuf>,

    /// Target selectors (repeatable). Examples: cjk (China/Japan/Korea), zh-cn|zh-hans|cn (China), zh-tw|zh-hant|tw (Taiwan), ru (Russia), ko (Korean), ko-kp|dprk (North Korea), ko-kr|rok (South Korea), ja (Japan), vi (Vietnam), th (Thailand), ar (Arabic-speaking regions), fa (Iran), he (Israel), hi (India), el (Greece), ur (Pakistan)
    #[arg(short, long, value_delimiter = ',')]
    lang: Vec<lang::Lang>,

    /// Add keyword mapping like lang=word (repeatable)
    #[arg(long)]
    keyword: Vec<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Show only lines with no detected labels
    #[arg(short = 'v', long)]
    invert_match: bool,

    /// Recurse into directories
    #[arg(short = 'r', long)]
    recursive: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct JsonFileResult {
    file: Option<String>,
    hits: BTreeMap<String, Vec<usize>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    misses: Vec<usize>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut langs = if args.lang.is_empty() {
        vec![
            lang::Lang::Cjk,
            lang::Lang::Ru,
            lang::Lang::Ko,
            lang::Lang::Ja,
            lang::Lang::Vi,
            lang::Lang::Th,
            lang::Lang::Ar,
            lang::Lang::Fa,
            lang::Lang::He,
            lang::Lang::Hi,
            lang::Lang::El,
            lang::Lang::Ur,
        ]
    } else {
        args.lang.clone()
    };
    if langs.iter().any(|l| *l == lang::Lang::Cjk) {
        if !langs.iter().any(|l| *l == lang::Lang::ZhHans) {
            langs.push(lang::Lang::ZhHans);
        }
        if !langs.iter().any(|l| *l == lang::Lang::ZhHant) {
            langs.push(lang::Lang::ZhHant);
        }
        if !langs.iter().any(|l| *l == lang::Lang::Ja) {
            langs.push(lang::Lang::Ja);
        }
    }
    if langs.iter().any(|l| *l == lang::Lang::Ko) {
        if !langs.iter().any(|l| *l == lang::Lang::KoKp) {
            langs.push(lang::Lang::KoKp);
        }
        if !langs.iter().any(|l| *l == lang::Lang::KoKr) {
            langs.push(lang::Lang::KoKr);
        }
    }
    langs.sort();
    langs.dedup();

    let keyword_map = lang::load_keywords(&args.keyword)?;

    if args.input.is_empty() {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        let mode = if args.invert_match {
            scanner::MatchMode::Unmatched
        } else {
            scanner::MatchMode::Matched
        };
        let lines = scanner::scan_collect(reader, &langs, &keyword_map, mode)?;
        if matches!(args.format, OutputFormat::Json) {
            let json = serde_json::to_string_pretty(&vec![JsonFileResult {
                file: None,
                hits: summarize_hits(&lines),
                misses: summarize_misses(&lines),
            }])
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            println!("{json}");
        } else {
            output_results(args.format, None, &lines, false, args.invert_match)?;
        }
        return Ok(());
    }

    let files = collect_files(&args.input, args.recursive)?;
    let show_file = files.len() > 1;
    let mut json_out: Vec<JsonFileResult> = Vec::new();

    for path in files {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(std::fs::File::open(&path)?));
        let mode = if args.invert_match {
            scanner::MatchMode::Unmatched
        } else {
            scanner::MatchMode::Matched
        };
        let lines = scanner::scan_collect(reader, &langs, &keyword_map, mode)?;
        let file_label = path.to_string_lossy().to_string();
        if matches!(args.format, OutputFormat::Json) {
            json_out.push(JsonFileResult {
                file: Some(file_label),
                hits: summarize_hits(&lines),
                misses: summarize_misses(&lines),
            });
        } else {
            output_results(
                args.format,
                Some(&file_label),
                &lines,
                show_file,
                args.invert_match,
            )?;
        }
    }

    if matches!(args.format, OutputFormat::Json) {
        let json = serde_json::to_string_pretty(&json_out)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        println!("{json}");
    }

    Ok(())
}

fn collect_files(paths: &[PathBuf], recursive: bool) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for path in paths {
        if path.is_dir() {
            if !recursive {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "directory input requires --recursive",
                ));
            }
            for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                }
            }
        } else {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn output_results(
    format: OutputFormat,
    file: Option<&str>,
    lines: &[scanner::LineHit],
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
    }
    Ok(())
}

fn summarize_hits(lines: &[scanner::LineHit]) -> BTreeMap<String, Vec<usize>> {
    let mut out: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for hit in lines {
        for lang in &hit.labels {
            let label = lang::lang_label(*lang).to_string();
            out.entry(label).or_default().push(hit.line_no);
        }
    }
    out
}

fn summarize_misses(lines: &[scanner::LineHit]) -> Vec<usize> {
    lines.iter()
        .filter(|hit| hit.labels.is_empty())
        .map(|hit| hit.line_no)
        .collect()
}
