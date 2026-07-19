# pyright: basic
"""Loader for the public-domain corpus bundled with corpus-tools."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


CHAR_TO_FILE = {"打": "da", "开": "kai", "发": "fa", "上": "shang", "下": "xia"}
REQUIRED_FIELDS = {
    "id",
    "text",
    "target_char",
    "source",
    "license",
    "access_date",
    "is_storable",
    "is_trainable",
    "is_publishable",
}


def load_bundled_fallback(target_char: str) -> list[dict[str, Any]]:
    """Load and validate bundled sentences for ``target_char``."""
    file_stem = CHAR_TO_FILE.get(target_char)
    if file_stem is None:
        raise ValueError(f"No bundled corpus for character: {target_char}")

    data_path = Path(__file__).parent / "data" / "fallback" / f"{file_stem}.jsonl"
    sentences: list[dict[str, Any]] = []
    with data_path.open(encoding="utf-8") as corpus_file:
        for line_number, raw_line in enumerate(corpus_file, start=1):
            line = raw_line.strip()
            if not line:
                continue
            record = json.loads(line)
            missing = REQUIRED_FIELDS - record.keys()
            if missing:
                raise ValueError(f"{data_path}:{line_number} missing fields: {sorted(missing)}")
            if record["target_char"] != target_char or target_char not in record["text"]:
                raise ValueError(f"{data_path}:{line_number} does not match target {target_char!r}")
            if record["is_publishable"] is not False:
                raise ValueError(f"{data_path}:{line_number} must not be publishable")
            sentences.append(record)
    return sentences
