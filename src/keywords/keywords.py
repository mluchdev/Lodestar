"""Ask a local Qwen (via Ollama) for synonyms of a topic, one keyword per line.

usage: python src/keywords/keywords.py "recurrent neural networks" [model]
"""
import json, re, sys, urllib.request

PROMPT = """List distinct terms researchers use for this topic: {topic}

Give synonyms, abbreviations, and names of specific well-known variants, \
architectures or methods that belong to it (for example, for "convolutional \
neural networks": cnn, convnet, resnet, vgg). Each term must be different from \
the others: do not repeat the topic with an added word. They will be searched \
for in paper abstracts. Return only a JSON object: {{"keywords": ["...", "..."]}}"""

topic = sys.argv[1]
model = sys.argv[2] if len(sys.argv) > 2 else "qwen3:4b"
req = urllib.request.Request(
    "http://localhost:11434/api/generate",
    json.dumps({"model": model, "prompt": PROMPT.format(topic=topic),
                "stream": False, "think": False, "format": "json"}).encode(),
)
answer = json.loads(json.load(urllib.request.urlopen(req))["response"])["keywords"]
words = dict.fromkeys(w.strip().lower() for w in [topic, *answer] if w.strip())
path = f"data/keywords_{re.sub(r'\W+', '_', topic.lower()).strip('_')}.txt"
with open(path, "w") as f:
    f.write("\n".join(words) + "\n")
print(f"{len(words)} keywords -> {path}")
