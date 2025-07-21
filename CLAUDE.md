# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
A Rust utility that processes geographic datasets (countries, states, cities) by splitting large JSON files into per-country files for efficient city-coordinate lookups.

## Architecture
- **Data Flow**: `data/raw/*.json` → processing → `data/generated/per-country/{id}_{code}.json`
- **Core Models**: `Country`, `State`, `City` structs with serde serialization
- **Processing**: Single-threaded JSON parsing and file generation

## Commands

### Build & Run
```bash
# Build and run the data processor
cargo run --release

# Development build
cargo build

# Check compilation without building
cargo check
```

### Development
```bash
# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings

# Run tests (when added)
cargo test

# Single test
cargo test test_name
```

## File Structure
- `src/main.rs`: Entry point with data processing logic
- `cargo.toml`: Dependencies (serde, serde_json)
- `data/raw/`: Input JSON files (countries.json, states+cities.json)
- `data/generated/per-country/`: Output per-country JSON files

## Key Patterns
- Uses `serde_json` for JSON parsing/serialization
- Error handling via `Result<T, Box<dyn std::error::Error>>`
- HashMap-based grouping for efficient data organization
- File I/O with `std::fs` for reading/writing JSON files