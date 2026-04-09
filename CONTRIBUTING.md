# Contributing

## Design Principles

`langscan` separates script-detection logic from language-specific data.

- `src/lang/*.rs` contains the core logic for each language or script family.
- This layer should hold Unicode range checks, script classification helpers, and lightweight matching rules.
- `src/lang/data/*.rs` contains supporting data used to refine classification.
- This layer should hold marker sets, regional keyword lists, and other tunable lookup data.
- Broad script-family labels and narrower language or regional labels are allowed to coexist.
- These labels are not required to be mutually exclusive.
- Example: `cjk` may coexist with `cn`, `tw`, or `ja`.
- Example: `ar` may coexist with `fa`.
- Example: `ko` may coexist with `dprk` or `rok`.

Examples:

- `src/lang/korean.rs` contains Hangul range detection.
- `src/lang/data/korean_kp.rs` and `src/lang/data/korean_kr.rs` contain DPRK and ROK keyword filters.
- `src/lang/chinese.rs` contains the matching functions, while `src/lang/data/chinese_*.rs` contains simplified and traditional marker sets.
- `src/lang/vietnamese.rs` contains the Unicode-based decision logic, while `src/lang/data/vietnamese.rs` contains Vietnamese-unique marker characters.

## Contribution Rules

- Keep detection logic in `src/lang/*.rs`.
- Keep tunable or language-specific data in `src/lang/data/*.rs`.
- Do not duplicate the same marker list or keyword list in both places.
- When adding a new language, add tests that cover both positive matches and likely false positives.
- Prefer conservative detection over broad matching when false positives would weaken forensic value.
- If a category is country- or region-specific, document that clearly in both the code and the README.
- If you add a narrower label under a broader script family, document the intended overlap explicitly.

## Testing

Before submitting changes, run:

```bash
cargo test
```
