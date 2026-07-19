#!/usr/bin/env python3
# pyright: basic
"""SenseFoundry corpus ingestion with scraper, wiki, and bundled fallbacks."""

from __future__ import annotations

import argparse
import json
import logging
import os
import sys
from pathlib import Path
from typing import Any

from fallback_corpus import load_bundled_fallback
from scraper import scrape_wiki_dump


logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

MIN_SENTENCES = 50
VALID_SOURCES = {"scraper", "wiki-dump", "bundled-fallback"}


def ingest(target_char: str, output_path: str) -> int:
    """Ingest one target character and return a process-compatible status."""
    source = os.environ.get("SEED_CORPUS_SOURCE", "bundled-fallback")
    if source not in VALID_SOURCES:
        logging.warning("Unknown corpus source %r; using bundled fallback", source)
        source = "bundled-fallback"

    sentences: list[dict[str, Any]] = []
    if source == "scraper":
        sentences = scrape_wiki_dump(target_char)
        if len(sentences) < MIN_SENTENCES:
            logging.warning(
                "Scraper returned %d sentences, falling back to bundled corpus",
                len(sentences),
            )
            source = "bundled-fallback"

    # A wiki dump needs explicit production configuration; local data is safe.
    if source in {"bundled-fallback", "wiki-dump"} or len(sentences) < MIN_SENTENCES:
        sentences = load_bundled_fallback(target_char)
        source = "bundled-fallback"
        logging.info(
            "Using bundled fallback corpus: %d sentences for %r",
            len(sentences),
            target_char,
        )

    if len(sentences) < MIN_SENTENCES:
        logging.error(
            "INSUFFICIENT DATA: only %d sentences for %r, need %d",
            len(sentences),
            target_char,
            MIN_SENTENCES,
        )
        print(
            json.dumps(
                {
                    "status": "insufficient_data",
                    "count": len(sentences),
                    "minimum": MIN_SENTENCES,
                }
            )
        )
        return 0

    lines = [json.dumps(sentence, ensure_ascii=False) for sentence in sentences]
    if output_path == "-":
        print("\n".join(lines))
    else:
        output = Path(output_path)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text("\n".join(lines) + "\n", encoding="utf-8")

    logging.info(
        "Ingested %d sentences for %r (source: %s)",
        len(sentences),
        target_char,
        source,
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", default="-")
    args = parser.parse_args()
    try:
        return ingest(args.target, args.output)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        logging.error("Fatal ingestion error: %s", exc)
        return 1


if __name__ == "__main__":
    sys.exit(main())
