from collections.abc import Sequence
from urllib.parse import urlparse

import httpx


class EmbeddingClient:
    """Client for an OpenAI-compatible cloud embedding API."""

    def __init__(self, base_url: str, api_key: str, model: str) -> None:
        base_url = base_url.rstrip("/")
        path = urlparse(base_url).path
        if path.endswith("/embeddings"):
            self.api_url = base_url
        elif path.endswith("/v1"):
            self.api_url = f"{base_url}/embeddings"
        elif path in ("", "/"):
            self.api_url = f"{base_url}/v1/embeddings"
        else:
            self.api_url = f"{base_url}/embeddings"
        self.api_key = api_key
        self.model = model

    async def embed(self, texts: Sequence[str]) -> list[list[float]]:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                self.api_url,
                headers={"Authorization": f"Bearer {self.api_key}"},
                json={"model": self.model, "input": list(texts)},
                timeout=30.0,
            )
            response.raise_for_status()
            data = response.json()

        return [item["embedding"] for item in data["data"]]
