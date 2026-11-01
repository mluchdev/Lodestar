"""Rank articles by embedding similarity of title+abstract to a topic description.

usage: uv run --with fastembed python src/sieve/rank.py data/ai.json "topic text" [top_n] [model]
"""
import json, sys
import numpy as np
from fastembed import TextEmbedding

path, topic = sys.argv[1], sys.argv[2]
top = int(sys.argv[3]) if len(sys.argv) > 3 else 10
model = TextEmbedding(sys.argv[4] if len(sys.argv) > 4 else "BAAI/bge-small-en-v1.5")

articles = json.load(open(path))
docs = model.embed(f"{a['title']}. {a['abstract']}" for a in articles)
vecs = np.array(list(docs))  # fastembed returns normalized vectors
query = next(iter(model.query_embed(topic)))
scores = vecs @ query
for i in np.argsort(-scores)[:top]:
    print(f"{scores[i]:.3f}  {articles[i]['title'][:100]}")
