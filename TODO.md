# TODO

The living plan for Lodestar. Keep it current; tick items off as they land.

## Decisions

- **Search:** the user gives a field/topic (free text) and gets the 10 best recent articles for it (last N days), per search.
- **Sources (first):** OpenAlex is the importer (metadata, abstracts, junk filtering by type). arXiv is used later for full text (HTML), via the `arxiv_id` stored with each article.
- **Junk filtering:** keep types article, preprint, review, conference-paper; drop retracted, no abstract, abstract under 30 words, housekeeping titles (erratum, editorial, ...). Mass-produced "template" genres and LLM-artifact text are only flagged, never dropped, and the top-10 stage skips `template-*` flags. Reviews go to the `syntheses` lane. Dropped works are logged. Rules: `src/fetch/junk.toml`. No local keyword re-check (it dropped real BiLSTM papers); OpenAlex's abstract search is the filter.
- **Full text:** avoid PDF parsing where possible: arXiv HTML (`arxiv.org/html/<id>`) or LaTeX source; JATS XML for Europe PMC / PMC / bioRxiv. Fallback for other sites: OpenDataLoader or EdgeParse, with Unpaywall to find the open-access PDF. PDFs only when no HTML/XML exists. Current focus: getting full text parsed.
- **Sieve (first level, must not drop real hits):** simple regex first, being very careful about false negatives; then an embedding model (BGE-M3 or similar) encodes abstracts and compares them to the user's topic. A fast ML tool may replace the regex.
- **Impact:** no hardcoded score. Impact is the jump from previous knowledge: how much a paper adds that was not known before (Mamba in late 2023 was a breakthrough for its genre even though transformers were still better). Results help, numerical results help more. Selection is automated, no LLM. Later: more objective signals.
- **Output:** the report is generated automatically as HTML or LaTeX, no LLM involved. The top 10 go in a kept, git-tracked directory `top10_<field>_<date>/` (the current one is `top10_field_date/`): PDF on disk (default) or just a link. Other PDFs are deleted.
- **LLM (later):** likely a local Gemma or Qwen, used only to explain why a solution was chosen.
- **Language:** Python is the core. A Rust CLI does fetching/API calls (and the regex sieve), async/multithreaded, and writes JSON into `data/` for Python to read. Code is split by function under `src/`.
- **Cache:** plain files in `data/` for now; a database is a TODO for later.
- **Hardware:** a laptop for now (CPU, keep it light), later a PC with an RTX GPU. Heavy jobs (e.g. embedding many abstracts) may be offloaded to Kaggle.

## Source limits (from research; verify before relying on them)

| Source | Page size | Rate limit | Pages for N days | Time |
|---|---|---|---|---|
| OpenAlex | 100 | 100 req/s + daily budget | ~500 | ~10-30 s |
| arXiv API | ~1000-2000 | 1 req / 3 s | ~25-50 | ~1.5-2.5 min |

OpenAlex without an API key: shared $0.10/day per IP, $0.001 per 200-result page; `page=` paging works (max 10000 results), cursor paging needs a free key.

Later candidates: Crossref, Semantic Scholar bulk, bioRxiv/medRxiv, Europe PMC, PubMed.

## Core (Breakthroughs lane, the default)

- [x] Rust fetcher: OpenAlex + junk filtering (`src/fetch`); RNN, 30 days gives about 2000 articles
- [x] Keyword synonyms from a local Qwen (`src/keywords`, `qwen3:4b` on the laptop)
- [ ] Full-text fetch and parsing (arXiv HTML first)
- [ ] Sieve: embeddings vs the user's topic (junk filtering is done in the fetcher)
- [ ] Select the top 10 with no LLM (see open questions)
- [ ] Auto-generated HTML/LaTeX report; top-10 directory (PDF or link)
- [ ] Simple CLI: topic, lane and N days in; report out

## Later

- [ ] Database for the cache (replaces plain files in `data/`)
- [ ] LLM "why chosen" explanations (Gemma or Qwen, local)
- [ ] More objective impact signals
- [ ] More sources: Crossref, Semantic Scholar, bioRxiv/medRxiv, Europe PMC, PubMed

## Other lanes

- [ ] Syntheses
- [ ] Engineering and applied
- [ ] Replications and negative results
- [ ] Datasets and benchmarks
- [ ] Open tools (GitHub Trending/search, Show HN, Product Hunt, Papers with Code)
- [ ] Cross-field surprises
- [ ] Contrarian and contested

## Open questions

- With no LLM and no hardcoded score, how do we measure "jump from previous knowledge"? Idea: embed each abstract and measure its distance from the earlier literature on the topic (novelty); results/numbers as a secondary signal. Needs a reference corpus of older papers.
- Embeddings: BGE-M3 on a laptop CPU is slow for thousands of abstracts. Plan: ONNX Runtime, same code on laptop CPU and on the PC's RTX via the CUDA provider; Kaggle for big batches. How do results come back (files in `data/`)?
- Tortured-phrase detection needs a fingerprint file (Problematic Paper Screener list); skipped until we have one.
- Should flagged template genres become drops after we see the stats?
- Which database, and cache lifetime; where the kept top-10 directory lives.
