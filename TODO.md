# TODO

The living plan for Lodestar. Keep it current; tick items off as they land.

## Decisions

- **Search:** the user gives a field/topic (free text) and gets the 10 best recent articles for it (last N days), per search.
- **Sources (first):** OpenAlex is the importer (metadata, abstracts, junk filtering by type). arXiv is used later for full text (HTML), via the `arxiv_id` stored with each article.
- **Junk filtering:** keep types article, preprint, review, conference-paper; drop retracted, no abstract, abstract under 30 words, housekeeping titles (erratum, editorial, ...). Mass-produced "template" genres and LLM-artifact text are only flagged, never dropped, and the top-10 stage skips `template-*` flags. Reviews go to the `syntheses` lane. Dropped works are logged. Rules: `src/fetch/junk.toml`. No local keyword re-check (it dropped real BiLSTM papers); OpenAlex's abstract search is the filter.
- **Full text:** avoid PDF parsing where possible: arXiv HTML (`arxiv.org/html/<id>`) or LaTeX source; JATS XML for Europe PMC / PMC / bioRxiv. Fallback for other sites: OpenDataLoader or EdgeParse, with Unpaywall to find the open-access PDF. PDFs only when no HTML/XML exists. Current focus: getting full text parsed.
- **Recall and sieve (embedding-first, no keyword guessing):** fetch broad slices of OpenAlex (e.g. the whole Computer Science field, by day), then embed abstracts (BGE-M3 or similar) and match them to a topic description. Why: keywords can't cover a topic like "LLM architectures". Measured on 30 days of OpenAlex: a "language model"/LLM seed query catches only 11% of Mamba, 3% of LSTM and 3% of GRU papers, because most never mention language models; OpenAlex's own subfield label catches 21% of Mamba and 11% of LSTM papers. Only embeddings can tell an LSTM language model from an LSTM electricity forecaster.
- **Qwen is not used for keywords:** `qwen3:4b` covered 5 of 20 required terms for "large language models" (missed RWKV, Mamba, SSM, LSTM, GRU, RNN, Qwen, DeepSeek) and invented noise ("paella", "opt"). A local LLM may only read abstracts (see LLM below).
- **Impact:** no hardcoded score. Impact is the jump from previous knowledge: how much a paper adds that was not known before (Mamba in late 2023 was a breakthrough for its genre even though transformers were still better). Results help, numerical results help more. Selection is automated, no LLM. Later: more objective signals.
- **Output:** the report is generated automatically as HTML or LaTeX, no LLM involved. The top 10 go in a kept, git-tracked directory `top10_<field>_<date>/` (the current one is `top10_field_date/`): PDF on disk (default) or just a link. Other PDFs are deleted.
- **Pipeline (agreed):** fetch a broad slice, drop junk (exclusion only, never "must contain"), cut with an embedding layer, then a local Qwen reads each remaining abstract and labels it: 0 = routine or not interesting enough, 1 = notable, 2 = exceptional. Articles labelled 1 or 2 go on for further inspection; the prompt must be mild enough to pass at least 10%. Keywords are only logged (`matched`), never used to filter.
- **LLM:** a local Qwen labels abstracts as above; later also explains why an article was chosen. Measured on the laptop (`qwen3:4b`, 4 threads, prompt prefix cached): about 10 s per article, so about 11 h for 3,849 articles. Accepted for now: the embedding layer should cut the input a lot, and the RTX PC is much faster. Peak RAM was 3.3 GB, which is too tight on the 7 GB laptop with a browser open.
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
- [x] Keyword synonyms from a local Qwen (`src/keywords`): works but unreliable for recall; superseded by the embedding-first plan, kept for narrow topics for now
- [ ] Full-text fetch and parsing (arXiv HTML first)
- [ ] Fetch mode without keywords: a whole field/subfield by day slices (each query is capped at 10000 results), cached per day in `data/`
- [ ] Embedding sieve: abstracts vs the topic description (junk filtering is done in the fetcher)
- [ ] Zero-shot NLI scoring (`MoritzLaurer/ModernBERT-base-zeroshot-v2.0`, CPU, about 0.6 s per paper per hypothesis): tested against real citations of 400 March-2023 AI papers (`data/cited_2023_03.json`, scores in `data/zs_cited.json`). Spearman with citations: "results reported" hypothesis +0.25, "breakthrough" +0.13, "new method" -0.02, combined +0.06, but the trivial baseline abstract length gets +0.31 and beats every hypothesis. Zero-shot NLI with hand-written hypotheses is not a good breakthrough detector; only the "reports measured results that outperform previous work" hypothesis shows some signal. Citations are a noisy proxy (small n, field and type effects).
- [ ] Qwen abstract labelling (0 = routine). Findings on 30 random AI articles with `qwen3:4b` (`data/labels_sample_v3.json`): about 9 s per article; hard labels are useless (25 of 30 got a 2, none a 1); ranking by token confidence puts hype first (a crank "E8 energy teleportation" paper and buzzword-heavy cryptosystem papers rank above real experiments) and plain business papers last. So a small model judges bombast, not substance. Next: ask it to extract the claimed result (metric, baseline, improvement) instead of rating, or use a bigger model on the RTX PC. Ollama memory grows per request: unload the model every 10 articles
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
- Embedding on the laptop crashed it (a large batch of ~3,800 abstracts in `src/sieve/rank.py` with fastembed, killed with exit 137): run embeddings on the RTX PC or Kaggle, or with small batches and a memory cap. BGE-M3 on a laptop CPU is slow for thousands of abstracts. Plan: ONNX Runtime, same code on laptop CPU and on the PC's RTX via the CUDA provider; Kaggle for big batches. How do results come back (files in `data/`)?
- Tortured-phrase detection needs a fingerprint file (Problematic Paper Screener list); skipped until we have one.
- Should flagged template genres become drops after we see the stats?
- OpenAlex free budget is $0.10/day (about 100 pages of 200 works). The Computer Science field is about 50,000 works per 30 days (250 pages), the AI subfield about 19,000 (96 pages). Without a key a 30-day CS fetch takes about 3 days of budget, or one free key ($1/day). Per-day caching makes either work; which field(s) first?
- Which database, and cache lifetime; where the kept top-10 directory lives.
