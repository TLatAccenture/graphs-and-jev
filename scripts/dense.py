#!/usr/bin/env python3
"""Recompute the DENSE table for Lab 1 in static/js/labs.js.

Cosine similarity (all-MiniLM-L6-v2) of each node's text in static/js/estate-graph.js
to each preset Lab 1 question, rounded to 3 decimal places. Prints one line of JSON
to paste over the DENSE constant.

    python3 -m venv /tmp/dense-venv && /tmp/dense-venv/bin/pip install sentence-transformers
    /tmp/dense-venv/bin/python scripts/dense.py
"""

import json
import re
from pathlib import Path

from sentence_transformers import SentenceTransformer, util

QUESTIONS = [
    "Which dashboards use KNA1.REGION?",
    "What feeds the regulatory return?",
    "Which workbook is the source of the price list?",
    "What does customer_sync write?",
]

graph = (
    Path(__file__).resolve().parent.parent / "static/js/estate-graph.js"
).read_text()
nodes = re.findall(r'\{id: "([^"]+)",.*?text: "([^"]+)"\}', graph)
assert len(nodes) == 15, f"expected 15 estate nodes, found {len(nodes)}"

model = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")
q_emb = model.encode(QUESTIONS, convert_to_tensor=True, normalize_embeddings=True)
n_emb = model.encode(
    [t for _, t in nodes], convert_to_tensor=True, normalize_embeddings=True
)
sim = util.cos_sim(q_emb, n_emb)

dense = {
    q: {nid: round(float(sim[i][j]), 3) for j, (nid, _) in enumerate(nodes)}
    for i, q in enumerate(QUESTIONS)
}
print(json.dumps(dense, separators=(",", ":")))
