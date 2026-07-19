from collections.abc import Sequence

import httpx


class EmbeddingClient:
    """Client for an OpenAI-compatible cloud embedding API."""

    def __init__(self, base_url: str, api_key: str, model: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.model = model

    async def embed(self, texts: Sequence[str]) -> list[list[float]]:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.base_url}/embeddings",
                headers={"Authorization": f"Bearer {self.api_key}"},
                json={"model": self.model, "input": list(texts)},
                timeout=30.0,
            )
            response.raise_for_status()
            data = response.json()

        return [item["embedding"] for item in data["data"]]
