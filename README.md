# City-Coordinates-Lookup Splitter

This utility takes your raw `countries.json` and `states+cities.json` (under `data/raw/`)
and splits them into per‐country JSON files (`{countryID}_{ISO2}.json`) in
`data/generated/per-country/`.

## Prerequisites

- Rust toolchain (1.56+)
- `serde` / `serde_json` (managed by Cargo)

## Usage

```bash
# From the repo root:
cargo run --release --manifest-path Cargo.toml
