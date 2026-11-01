# Lodestar

An agentic system that finds the most impactful research articles from the last N days, so a researcher stays on the edge of new work.

## Idea

1. **Collect** recent articles from research sources.
2. **Sieve** them with regex (keywords, topics), so the LLM never sees thousands of papers.
3. **Rank** the survivors with an LLM agent by likely impact and relevance.

The project is young and its direction is open.

## Layout

- `src/` – code
- `data/` – local data (git-ignored)
- `lodestar.tex` – short write-up; build with `pdflatex lodestar.tex`

## Platform

Built for Linux. Windows may work, but it is not guaranteed.
