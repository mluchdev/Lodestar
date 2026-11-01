# Lodestar

An agentic system that finds the most impactful research articles from the last N days, so a researcher stays on the edge of new work.

## Idea

1. **Collect** recent articles from research sources.
2. **Sieve** them (careful regex, then embeddings against the user's topic) down to a small set.
3. **Rank** the survivors by likely impact and relevance, and generate an HTML/LaTeX report of the top 10. No LLM is needed; a local one may later explain the choices.

## Lanes

Each search runs in a lane, a different kind of "worth reading". The first is the default.

1. **Breakthroughs** (default) – original, revolutionary research
2. **Syntheses** – reviews, surveys, meta-analyses
3. **Engineering and applied** – new technology
4. **Replications and negative results**
5. **Datasets and benchmarks**
6. **Open tools** – general, not only scientific. Sources: GitHub Trending, GitHub search API (new repos by star velocity), Hacker News "Show HN", Product Hunt, Papers with Code. Scored on star growth, commit activity, docs quality, and whether it is usable rather than a demo.
7. **Cross-field surprises**
8. **Contrarian and contested**

The project is young and its direction is open.

## Layout

- `src/` – code
- `TODO.md` – the plan; the heart of the project
- `data/` – local data (git-ignored)
- `lodestar.tex` – short write-up; build with `pdflatex lodestar.tex`

## Platform

Built for Linux. Windows may work, but it is not being tested on that platform.
