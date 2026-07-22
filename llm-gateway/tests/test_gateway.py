import asyncio
import sys
from pathlib import Path

from fastapi.testclient import TestClient

sys.path.insert(0, str(Path(__file__).parents[1]))

from app.main import app, get_gateway  # pyright: ignore[reportImplicitRelativeImport]  # noqa: E402
from app.gateway import LlmGateway  # pyright: ignore[reportImplicitRelativeImport]  # noqa: E402


class RecordingGateway:
    api_key: str = "test-key"

    def __init__(self) -> None:
        self.prompt_arguments: tuple[str, str, list[str]] | None = None

    async def draft_definition(
        self, headword: str, pos: str, evidence_texts: list[str]
    ) -> str:
        self.prompt_arguments = (headword, pos, evidence_texts)
        return "A corpus-grounded definition."

    async def reachable(self) -> bool:
        return True


def test_evidence_constraint_enforced() -> None:
    gateway = RecordingGateway()
    app.dependency_overrides[get_gateway] = lambda: gateway
    client = TestClient(app)

    response = client.post(
        "/draft-definition",
        json={
            "headword": "bank",
            "pos": "noun",
            "evidence_ids": ["card-1", "card-2"],
            "evidence_items": [
                {"id": "card-1", "text": "The river overflowed its bank."},
                {"id": "card-2", "text": "They sat on the grassy bank."},
                {"id": "unused", "text": "This must not reach the model."},
            ],
        },
    )

    assert response.status_code == 200
    assert response.json()["evidence_ids_used"] == ["card-1", "card-2"]
    assert gateway.prompt_arguments == (
        "bank",
        "noun",
        ["The river overflowed its bank.", "They sat on the grassy bank."],
    )
    app.dependency_overrides.clear()


def test_empty_evidence_returns_422() -> None:
    client = TestClient(app)
    response = client.post(
        "/draft-definition",
        json={"headword": "bank", "pos": "noun", "evidence_ids": [], "evidence_items": []},
    )
    assert response.status_code == 422


def test_chat_completion_allows_full_model_latency(monkeypatch) -> None:
    class Response:
        content = b'{"choices":[{"message":{"content":"definition"}}]}'

        def raise_for_status(self) -> None:
            return None

    class RecordingClient:
        timeout: float | None = None

        def __init__(self, *, timeout: float) -> None:
            RecordingClient.timeout = timeout

        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc, traceback) -> None:
            return None

        async def post(self, *args, **kwargs) -> Response:
            return Response()

    monkeypatch.setenv("LLM_MODEL", "configured-model")
    monkeypatch.setattr("app.gateway.httpx.AsyncClient", RecordingClient)

    definition = asyncio.run(
        LlmGateway(
            api_key="test-key",
            api_url="https://example.test/v1/chat/completions",
        ).draft_definition("word", "noun", ["evidence"])
    )

    assert definition == "definition"
    assert RecordingClient.timeout == 60.0
