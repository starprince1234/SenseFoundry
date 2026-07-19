# pyright: basic
import json
import os
import sys
from pathlib import Path

import pytest


CORPUS_TOOLS_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(CORPUS_TOOLS_DIR))


def test_bundled_fallback_has_min_sentences() -> None:
    """Each target character has at least the frozen baseline."""
    from fallback_corpus import load_bundled_fallback

    for char in ["打", "开", "发", "上", "下"]:
        sentences = load_bundled_fallback(char)
        assert len(sentences) >= 50, f"{char!r} only has {len(sentences)} sentences"


@pytest.mark.parametrize("char", ["打", "开", "发", "上", "下"])
def test_each_sentence_is_compliant(char: str) -> None:
    from fallback_corpus import REQUIRED_FIELDS, load_bundled_fallback

    for sentence in load_bundled_fallback(char):
        assert REQUIRED_FIELDS <= sentence.keys()
        assert char in sentence["text"]
        assert sentence["target_char"] == char
        assert sentence["source"] == "bundled_fallback"
        assert sentence["license"] == "public_domain"
        assert sentence["is_storable"] is True
        assert sentence["is_trainable"] is True
        assert sentence["is_publishable"] is False


def test_seed_senses_not_authoritative_or_publishable() -> None:
    from seed_senses import SEED_SENSES

    for senses in SEED_SENSES.values():
        for sense in senses:
            assert sense["is_authoritative"] is False
            assert sense["is_publishable"] is False
            assert sense["source_kind"] == "internal_seed"


def test_scraper_failure_does_not_hang() -> None:
    from scraper import scrape_wiki_dump

    result = scrape_wiki_dump("打")
    assert isinstance(result, list)
    assert result == []


def test_ingest_falls_back_and_writes_valid_jsonl(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from ingest import ingest

    monkeypatch.setenv("SEED_CORPUS_SOURCE", "scraper")
    output = Path(__file__).with_name("ingest-output.tmp")
    try:
        assert ingest("打", str(output)) == 0
        records = [
            json.loads(line) for line in output.read_text(encoding="utf-8").splitlines()
        ]
        assert len(records) >= 50
        assert all(record["target_char"] == "打" for record in records)
    finally:
        output.unlink(missing_ok=True)


def test_unknown_character_is_rejected() -> None:
    from fallback_corpus import load_bundled_fallback

    with pytest.raises(ValueError, match="No bundled corpus"):
        load_bundled_fallback("中")


def test_source_environment_does_not_leak(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("SEED_CORPUS_SOURCE", raising=False)
    assert "SEED_CORPUS_SOURCE" not in os.environ
