use clap::Parser;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use walkdir::WalkDir;

mod lang;
mod output;
mod scanner;

pub use output::OutputFormat;
use output::{
    collect_serialized_file_results, flush_serialized_file_results, print_counts,
    print_counts_for_file, print_serialized_results, print_total_counts, JsonFileResult,
    YamlFileResult,
};
pub use scanner::{scan_collect, LineHit, MatchMode};

#[derive(Parser, Debug)]
#[command(name = "langscan")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Scan lines for uncommon scripts (CJK, Cyrillic) and language hints")]
struct Args {
    /// Input file or directory (defaults to stdin)
    input: Vec<PathBuf>,

    /// Target selectors (repeatable). Examples: cjk (China/Japan/Korea), zh-cn|zh-hans|cn (China), zh-tw|zh-hant|tw (Taiwan), ru (Russia), ko (Korean), ko-kp|dprk (North Korea), ko-kr|rok (South Korea), ja (Japan), vi (Vietnam), th (Thailand), tr (Turkey), ar (Arabic-speaking regions), fa (Iran), he (Israel), hi (India), el (Greece), ur (Pakistan)
    #[arg(short, long, value_delimiter = ',')]
    lang: Vec<lang::Lang>,

    /// Add keyword mapping like lang=word (repeatable)
    #[arg(long)]
    keyword: Vec<String>,

    /// Load keyword mappings from files with one lang=word entry per line
    #[arg(long = "keyword-file")]
    keyword_files: Vec<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Show only lines with no detected labels
    #[arg(short = 'v', long)]
    invert_match: bool,

    /// Recurse into directories
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Show match statistics
    #[arg(long)]
    stats: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut langs = if args.lang.is_empty() {
        vec![
            lang::Lang::Ar,
            lang::Lang::Cjk,
            lang::Lang::El,
            lang::Lang::Fa,
            lang::Lang::He,
            lang::Lang::Hi,
            lang::Lang::Ja,
            lang::Lang::Ko,
            lang::Lang::Pl,
            lang::Lang::Ru,
            lang::Lang::Th,
            lang::Lang::Tr,
            lang::Lang::Uk,
            lang::Lang::Ur,
            lang::Lang::Vi,
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

    let keyword_entries = load_keyword_entries(&args.keyword, &args.keyword_files)?;
    let keyword_map = lang::load_keywords(&keyword_entries)?;

    if args.input.is_empty() {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        let mode = if args.invert_match {
            MatchMode::Unmatched
        } else {
            MatchMode::Matched
        };
        let lines = scan_collect(reader, &langs, &keyword_map, mode)?;
        if args.stats {
            print_counts(&lines);
        } else {
            match args.format {
                OutputFormat::Text => {
                    output::output_results(args.format, None, &lines, false, args.invert_match)?;
                }
                OutputFormat::Json | OutputFormat::JsonLines | OutputFormat::Yaml => {
                    print_serialized_results(args.format, None, &lines)?;
                }
            }
        }
        return Ok(());
    }

    let files = collect_files(&args.input, args.recursive)?;
    let show_file = files.len() > 1;
    let need_total_stats = args.stats && files.len() > 1;
    let mode = if args.invert_match {
        MatchMode::Unmatched
    } else {
        MatchMode::Matched
    };

    let results_per_file: Vec<io::Result<(PathBuf, Vec<LineHit>)>> = files
        .par_iter()
        .map(|path| {
            let reader: Box<dyn BufRead> = Box::new(BufReader::new(std::fs::File::open(path)?));
            let hits = scan_collect(reader, &langs, &keyword_map, mode)?;
            Ok((path.clone(), hits))
        })
        .collect();
    let results_by_file: Vec<(PathBuf, Vec<LineHit>)> = results_per_file
        .into_iter()
        .collect::<io::Result<Vec<_>>>()?;

    let mut json_out: Vec<JsonFileResult> = Vec::new();
    let mut yaml_out: Vec<YamlFileResult> = Vec::new();
    let mut all_lines: Vec<LineHit> = Vec::new();

    for (path, lines) in &results_by_file {
        let file_label = path.to_string_lossy().to_string();
        if args.stats {
            print_counts_for_file(&file_label, lines);
        } else {
            match args.format {
                OutputFormat::Text => {
                    output::output_results(
                        args.format,
                        Some(&file_label),
                        lines,
                        show_file,
                        args.invert_match,
                    )?;
                }
                OutputFormat::Json | OutputFormat::Yaml => {
                    collect_serialized_file_results(
                        args.format,
                        &file_label,
                        lines,
                        &mut json_out,
                        &mut yaml_out,
                    );
                }
                OutputFormat::JsonLines => {
                    print_serialized_results(args.format, Some(&file_label), lines)?;
                }
            }
        }
        if need_total_stats {
            all_lines.extend(lines.iter().cloned());
        }
    }

    if !args.stats {
        flush_serialized_file_results(args.format, &json_out, &yaml_out)?;
    }
    if need_total_stats {
        print_total_counts(&all_lines);
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

fn load_keyword_entries(cli_keywords: &[String], keyword_files: &[PathBuf]) -> io::Result<Vec<String>> {
    let mut entries = cli_keywords.to_vec();

    for path in keyword_files {
        let reader = BufReader::new(File::open(path)?);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            entries.push(trimmed.to_string());
        }
    }

    Ok(entries)
}
