# langscan

A small tool that scans a collection of strings to find those in different locales

## Use Cases

- Scan MFT-derived file paths, ADS names, or recovered text during forensics to identify files created or labeled in foreign-language scripts.
- Scan `strings` output from PE files, DLLs, shellcode, or unpacked malware to spot language hints tied to operators, tooling, or victim targeting.
- Review extracted logs, command history, or dropped text artifacts for Chinese, Russian, Korean, Japanese, Arabic, Persian, Hebrew, Urdu, Thai, Hindi, Greek, or Vietnamese content.
- Triage directories recursively to surface multilingual filenames or embedded text before deeper reverse engineering or attribution work.

## Usage

```bash
# Scan a file (default: cjk/ru/ko/ja/vi/th/ar/fa/he/hi/el/ur)
langscan ./input.txt

# Read from stdin
cat input.txt | langscan

# Limit languages
langscan --lang ru --lang cjk input.txt

# Show only lines with no matches
langscan -v --lang ja input.txt

# Include Korean regional tags (ko adds dprk/rok automatically)
langscan --lang ko input.txt

# Extra languages
langscan --lang vi --lang th --lang ar --lang fa --lang he --lang hi --lang el --lang ur input.txt

```

## Output

Text output (default) uses:

```
[L3:cjk,ja] line text with ANSI highlights
```

When scanning multiple files, the file path is included:

```
[path/to/file:L3:cjk,ja] line text with ANSI highlights
```

JSON output groups line numbers per label:

```bash
langscan --format json input.txt
```

```json
[
  {
    "file": "input.txt",
    "hits": {
      "ja": [2, 5],
      "ru": [7]
    }
  }
]
```

## CLI

```
langscan [<input>...]

Options:
  -l, --lang <LANG>     Target selectors (repeatable or comma-separated)
  -v, --invert-match    Show only lines with no detected labels
      --keyword <K=V>   Add keyword mapping like lang=word (repeatable)
      --format <FMT>    Output format: text|json
  -r, --recursive       Recurse into directories
```

Accepted `--lang` values:
- `ar`: Arabic, Arabic-speaking regions
- `cjk`: CJK ideographs across China, Japan, and Korea
- `el`: Greek, Greece
- `fa`, `fas`, `per`, `persian`, `farsi`: Persian, Iran
- `he`, `heb`, `hebrew`: Hebrew, Israel
- `hi`: Hindi / Devanagari, India
- `ja`: Japanese, Japan
- `ko`: Korean, general
- `ko-kr`, `rok`: Korean, South Korea
- `ko-kp`, `dprk`: Korean, North Korea
- `ru`: Russian / Cyrillic, Russia-oriented
- `th`: Thai, Thailand
- `ur`, `urd`, `urdu`: Urdu, Pakistan
- `vi`: Vietnamese, Vietnam
- `zh-cn`, `zh-hans`, `cn`: Simplified Chinese, China
- `zh-tw`, `zh-hant`, `tw`: Traditional Chinese, Taiwan

Built-in `cn` / `tw` markers are derived from Unicode Unihan simplified/traditional variant data.
Japanese Joyo Kanji are excluded from those built-in marker tables to keep the labels conservative.

## Output Labels

- Labels are not always mutually exclusive.
- Broad script-family labels can appear together with narrower country- or language-oriented labels.
- Example: `cjk` can appear together with `cn`, `tw`, or `ja`.
- Example: `ar` can appear together with `fa`.
- Example: `ko` can appear together with `dprk` or `rok`.
- This is intentional. The broad label indicates the script family, while the narrower label indicates a more specific language or regional signal.

## Test data
A small sample file is included at `testdata/sample.txt`.

## From GitHub Releases

Pre-compiled standalone binaries are available.
Each release ships direct executable files for Linux and Windows, not zip-only bundles.

```bash
chmod +x ./langscan-linux-x86_64
./langscan-linux-x86_64 {{options...}}
```

```powershell
.\langscan-windows-x86_64.exe {{options...}}
```

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for the project layout and design principles.
The source code for `langscan` is hosted on GitHub at <https://github.com/sumeshi/langscan>.
Contributions, forks, and reviews are encouraged. Please open issues and submit feature requests as needed.
If you are fluent in any of the supported languages or regional variants, contributions to improve marker sets, keyword lists, and false-positive handling are especially welcome.

## License

`langscan` is released under the MIT License.
