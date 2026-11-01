# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Early stage. Working so far: keyword synonyms from a local Qwen (`src/keywords`) and an OpenAlex importer with junk filtering (`src/fetch`). Not built yet: ranking, full-text fetch, the report. Plan and open questions are in `TODO.md`.

## What it is

An agentic system that finds the most impactful research articles from the last N days: collect, sieve, rank. Searches run in "lanes" (Breakthroughs is the default). See `README.md`.

## Commands

- `python src/keywords/keywords.py "<topic>" [model]` asks a local Qwen (Ollama) for synonyms, writes `data/keywords_<topic>.txt`.
- `cargo build --release --manifest-path src/fetch/Cargo.toml`, then `src/fetch/target/release/lodestar-fetch --keywords FILE [--days N] [--max M] [--out FILE] [--junk FILE]` fetches OpenAlex works whose abstract matches a keyword into `data/articles.json` (dropped ones are logged in `data/dropped.json`). Junk rules live in `src/fetch/junk.toml`. `cargo test --manifest-path src/fetch/Cargo.toml` runs the tests.
- OpenAlex needs no API key for this (plain `page=` paging, max 10000 results). `OPENALEX_API_KEY` is optional.

## Where things live

- `TODO.md` is the plan and the heart of the project. Keep it current; tick items off as they land.
- `src/` code, `data/` local data (git-ignored), `lodestar.tex` write-up (`pdflatex lodestar.tex`; PDFs and LaTeX build files are git-ignored).

## Conventions

- Linux (Arch) only; Windows is untested.
- Keep it minimal: use the standard library and built-in tools first, and add a dependency only when clearly necessary.
- Keep config files short and readable.
- Python is the core; a Rust CLI does fetching/API calls and writes JSON into `data/` for Python. Split code by function under `src/`.
- Dev machine is a laptop for now (keep things CPU-light); a PC with an RTX and Kaggle offloading come later.
- Avoid PDF parsing: use arXiv HTML or LaTeX source, and JATS XML for biomedical sources. Use PDFs only when no HTML/XML exists. PDFs are only kept for the top 10 results (or a link instead).
- Selection and the HTML/LaTeX report are fully automated, with no LLM. A local LLM (Gemma/Qwen) is a later add-on that only explains why an article was chosen.
- No hardcoded breakthrough score. Impact is the jump from previous knowledge, with results (especially numerical) as a plus.
- A breakthrough is a new, surprising solution backed by claimed results. Recall comes from broad fetches plus an embedding sieve, not from keyword lists (keywords and Qwen-made synonyms measurably miss most of a broad topic; see `TODO.md`). False negatives matter more than false positives.
- When unsure, don't filter: a false negative (a real breakthrough dropped) is worse than a false positive. Junk rules drop only deterministic junk and merely flag the rest.
- Dev stage: no defensive error handling in inner code (`unwrap`, let it crash); only check errors where the outside world is involved (API requests, CLI usage).
- Decisions and open questions are in `TODO.md`.
