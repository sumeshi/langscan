# Contributing

## Design Principles

`langscan` separates script-detection logic from language-specific data.

- `src/lang/*.rs` contains the core logic for each language or script family.
- This layer should hold Unicode range checks, script classification helpers, and lightweight matching rules.
- `src/lang/markers/*.rs` contains supporting data used to refine classification.
- This layer should hold marker sets, regional keyword lists, and other tunable lookup data.
- `src/scanner/*.rs` contains scan pipeline pieces such as flag collection, match evaluation, and highlighting.
- Broad script-family labels and narrower language or regional labels are allowed to coexist.
- These labels are not required to be mutually exclusive.
- Example: `cjk` may coexist with `cn`, `tw`, or `ja`.
- Example: `ar` may coexist with `fa`.
- Example: `ko` may coexist with `dprk` or `rok`.

Examples:

- `src/lang/korean.rs` contains Hangul range detection.
- `src/lang/markers/korean_kp.rs` and `src/lang/markers/korean_kr.rs` contain DPRK and ROK keyword filters.
- `src/lang/chinese.rs` contains the matching functions, while `src/lang/markers/chinese_*.rs` contains simplified and traditional marker sets.
- `src/lang/latin.rs` contains shared Latin-script helpers, while `src/lang/markers/latin.rs` contains shared Latin marker ranges used by language-specific logic.
- `src/lang/vietnamese.rs` contains the Unicode-based decision logic, while `src/lang/markers/vietnamese.rs` contains Vietnamese-unique marker characters.
- `src/scanner/flags.rs` defines the scan plan and per-line script flags.
- `src/scanner/detect.rs` maps collected flags and keyword hints to output labels.
- `src/scanner/highlight.rs` handles ANSI highlighting for text output.

## Contribution Rules

- Keep detection logic in `src/lang/*.rs`.
- Keep tunable or language-specific data in `src/lang/markers/*.rs`.
- Keep scan orchestration in `src/scanner/*.rs` rather than growing `main.rs`.
- Do not duplicate the same marker list or keyword list in both places.
- When adding a new language, add tests that cover both positive matches and likely false positives.
- Prefer conservative detection over broad matching when false positives would weaken forensic value.
- If a category is country- or region-specific, document that clearly in both the code and the README.
- If you add a narrower label under a broader script family, document the intended overlap explicitly.

## Local Workflow

Before submitting changes, run:

```bash
cargo test
```

If you changed the CLI surface, also review:

```bash
cargo run --quiet -- --help
```

## Testing

Run `cargo test` before submitting changes. If you touched CLI behavior or documentation, also verify `cargo run --quiet -- --help` matches the docs.
