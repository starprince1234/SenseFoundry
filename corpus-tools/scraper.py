# pyright: basic
"""License-aware corpus acquisition with a deterministic local fallback."""

from __future__ import annotations

import logging
import os
from typing import Any

from fallback_corpus import load_bundled_fallback


logger = logging.getLogger(__name__)

SEED_CORPUS_SOURCE = os.environ.get("SEED_CORPUS_SOURCE", "bundled-fallback")
MIN_SENTENCES = 50


def scrape_wiki_dump(target_char: str) -> list[dict[str, Any]]:
    """Attempt CC-BY-SA Wikipedia ingestion, returning promptly on failure.

    Network acquisition is intentionally disabled until a configured dump URL,
    checksum, attribution record, and bounded timeout are available.
    """
    try:
        raise RuntimeError("Wiki dump not configured")
    except Exception as exc:  # The caller must always be able to fall back.
        logger.warning(
            "Scraper failed for %s: %s. Falling back to bundled corpus.",
            target_char,
            exc,
        )
        return []


__all__ = [
    "MIN_SENTENCES",
    "SEED_CORPUS_SOURCE",
    "load_bundled_fallback",
    "scrape_wiki_dump",
]
